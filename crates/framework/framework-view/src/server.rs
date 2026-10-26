//! Native page registration through ordinary server routes and the existing lifecycle.
//!
//! Named loaders return owned public data with a Send future. Page identity claims
//! use Config-phase named services, so duplicates fail build before publication.
//! No runtime render registry, reactive Owner or global builder map is introduced.

use crate::{
    PageId, PageLoad, PageRenderer, PageSnapshot, PublicLocation, PublicStatus, ViewError,
};
use stackpress_lib::{
    DeliveryState, Error, EventPattern, Flow, HandlerFuture, Invocation, ListenerId, Request,
    Response, Result, handler, queue::task,
};
use stackpress_server::{ServerBuilder, StartupContext};
use std::{future::Future, sync::Arc};

/// A checked GET page definition with explicit public location projection.
pub struct PageSpec<T, L> {
    /// Native method/path matching stays under the existing server adapter.
    path: String,
    /// Catalog identity; duplicate claims are rejected in Config startup.
    id: PageId,
    /// Named Send loader; it owns no live invocation between calls.
    loader: L,
    /// Optional allowlisted query/path construction from the loaded public data.
    location: Option<Arc<LocationProjection<T>>>,
}

/// The public location projector cannot implicitly see request/session contents.
type LocationProjection<T> =
    dyn Fn(&T) -> std::result::Result<PublicLocation, ViewError> + Send + Sync;

impl<T: Send + 'static, L: PageLoader<T>> PageSpec<T, L> {
    /// Check the native GET pattern and page identity before queuing startup work.
    ///
    /// Without [`Self::location`], the canonical public location contains only the
    /// actual pathname. Query data is never forwarded implicitly.
    ///
    /// # Errors
    /// Returns checked foundation pattern errors or invalid catalog identity.
    pub fn get(path: &str, id: &str, loader: L) -> Result<Self> {
        EventPattern::express(path, '/')?;
        PublicLocation::new(path).map_err(Error::from_source)?;
        if path.contains('?') {
            return Err(Error::invalid("page route cannot contain a query"));
        }
        let id = PageId::try_from(id.to_owned()).map_err(Error::from_source)?;
        Ok(Self {
            path: path.into(),
            id,
            loader,
            location: None,
        })
    }

    /// Explicitly select public query/path fields from the loaded data.
    pub fn location(
        mut self,
        projector: impl Fn(&T) -> std::result::Result<PublicLocation, ViewError> + Send + Sync + 'static,
    ) -> Self {
        self.location = Some(Arc::new(projector));
        self
    }
}

/// Register a page as an ordinary GET handler with no second top-level lifecycle.
pub trait ServerBuilderViewExt {
    /// Queue one checked page and Config-phase identity claim.
    ///
    /// Duplicate IDs fail `build()` before runtime publication, including claims
    /// from different selected plugins. Native duplicate route listeners retain
    /// the server's normal all-matching semantics. Discard a builder after any
    /// registration error; registration is not a transaction across server queues.
    ///
    /// # Errors
    /// Returns invalid page/renderer policy and ordinary server registration errors.
    fn page<T, L, R>(&mut self, spec: PageSpec<T, L>, renderer: R) -> Result<ListenerId>
    where
        T: Send + 'static,
        L: PageLoader<T>,
        R: PageRenderer<T>;
}

impl ServerBuilderViewExt for ServerBuilder {
    /// Register an owned loader/renderer without retaining reactive or request state.
    fn page<T, L, R>(&mut self, spec: PageSpec<T, L>, renderer: R) -> Result<ListenerId>
    where
        T: Send + 'static,
        L: PageLoader<T>,
        R: PageRenderer<T>,
    {
        renderer.contract().validate().map_err(Error::from_source)?;
        let claim = format!("framework-view::page::{}", spec.id.as_str());
        self.on_config(task(move |start: &mut StartupContext| {
            let claim = claim.clone();
            Box::pin(async move {
                // A named service is a build-time singleton claim, never a mutable render cache.
                start.register_named(&claim, Arc::new(PageIdentity))?;
                Ok(Flow::Continue)
            })
        }))?;
        let renderer = Arc::new(renderer);
        let loader = Arc::new(spec.loader);
        self.get(
            &spec.path,
            handler(move |req, res, inv| {
                let renderer = Arc::clone(&renderer);
                let loader = Arc::clone(&loader);
                let id = spec.id.clone();
                let location = spec.location.clone();
                Box::pin(async move {
                    // A prior all-matching listener may have finalized this exact response.
                    if res.delivery_state() != DeliveryState::Open {
                        return Ok(Flow::Continue);
                    }
                    let data = match loader_contract::Erased::invoke(loader.as_ref(), req, res, inv)
                        .await?
                    {
                        PageLoad::Abort => return Ok(Flow::Abort),
                        PageLoad::Ready(data) => data,
                    };
                    // A loader can stop/send output deliberately; never render or retry afterward.
                    if res.delivery_state() != DeliveryState::Open {
                        return Ok(Flow::Continue);
                    }
                    let location = match location {
                        Some(projector) => projector(&data),
                        None => PublicLocation::new(req.url.path()),
                    }
                    .map_err(Error::from_source)?;
                    let status = PublicStatus::new(
                        res.status().map_or(200, |status| status.code),
                        res.error().map(str::to_owned),
                    )
                    .map_err(Error::from_source)?;
                    let status_code = status.code();
                    let contract = renderer.contract();
                    let snapshot = PageSnapshot {
                        revision: contract.revision,
                        build: contract.build.clone(),
                        page: id,
                        location: location.into(),
                        locale: contract.locale.clone(),
                        theme: contract.theme.clone(),
                        status,
                        data,
                    };
                    // Rendering receives owned public data only; the host keeps its original pair.
                    let page = renderer
                        .render(snapshot)
                        .await
                        .map_err(Error::from_source)?;
                    res.html(page.into_html())?;
                    // Foundation HTML staging selects default OK. Restore the public page
                    // status explicitly so a complete not-found page remains HTTP 404.
                    res.status_code(status_code, None)?;
                    Ok(Flow::Continue)
                })
            }),
        )
    }
}

/// Presence in a Config-phase named slot is the page identity reservation.
struct PageIdentity;

/// Lifetime-family adaptation follows the foundation's checked Send handler contract.
mod loader_contract {
    use super::*;
    /// A named loader's concrete future for one common invocation borrow.
    pub trait LoaderFn<'a, T> {
        /// Public data is owned; the future itself remains Send and borrow-bounded.
        type Future: Future<Output = Result<PageLoad<T>>> + Send + 'a;
        /// Invoke without extending the request/response/context lifetime.
        fn invoke(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            inv: Invocation<'a>,
        ) -> Self::Future;
    }
    impl<'a, T, F, Fut> LoaderFn<'a, T> for F
    where
        F: Fn(&'a mut Request, &'a mut Response, Invocation<'a>) -> Fut,
        Fut: Future<Output = Result<PageLoad<T>>> + Send + 'a,
    {
        type Future = Fut;
        /// Return the original Send future family directly.
        fn invoke(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            inv: Invocation<'a>,
        ) -> Fut {
            self(req, res, inv)
        }
    }
    /// Sealed object-safe invocation, never an escape hatch around Send.
    pub trait Erased<T>: Send + Sync {
        /// Erase only a future already checked at this exact lifetime.
        fn invoke<'a>(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            inv: Invocation<'a>,
        ) -> HandlerFuture<'a, Result<PageLoad<T>>>;
    }
    impl<T, F> Erased<T> for F
    where
        F: for<'a> LoaderFn<'a, T> + Send + Sync + 'static,
    {
        /// Boxing pins the checked future without changing its lifetime or thread contract.
        fn invoke<'a>(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            inv: Invocation<'a>,
        ) -> HandlerFuture<'a, Result<PageLoad<T>>> {
            Box::pin(LoaderFn::invoke(self, req, res, inv))
        }
    }
}

/// A named asynchronous page loader whose actual returned future is Send.
pub trait PageLoader<T>: loader_contract::Erased<T> + 'static {}
impl<T, F> PageLoader<T> for F where
    F: for<'a> loader_contract::LoaderFn<'a, T> + Send + Sync + 'static
{
}
