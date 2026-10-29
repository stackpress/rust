//! Stackpress page framing around one independently supervised Leptos byte stream.
use super::{
    render::{limits as render_limits, map_error, require_executor},
    safety::escape_html,
};
use crate::{PageAssets, PageContract, PageSnapshot, PublicLocation, ViewError};
use futures::Stream;
use leptos::prelude::*;
use leptos_meta::ServerMetaContext;
use serde::Serialize;
/// Finite generic stream policy, with unchanged native defaults.
pub use stackpress_leptos::stream::StreamLimits;
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
const FAILURE: &str = "<template id=\"stackpress-stream-failed\"></template>";
/// Fixed external recovery-status script; the host explicitly admits and serves it.
pub const STREAM_STATUS_SCRIPT: &str = include_str!("stream_status.js");
/// Checked optional renderer; it does not alter either buffered renderer trait.
///
/// # Examples
/// ```
/// use leptos::prelude::*;
/// use stackpress_framework_view::leptos::stream::{StreamLeptosRenderer, StreamLimits};
/// use stackpress_framework_view::{BasePath, PageAssets, PageContract, PageSnapshot, PublicLocation};
/// let contract = PageContract::new(1, "app-build")?;
/// let assets = PageAssets {
///     theme_bootstrap: None,
///     build: "app-build".into(), base: BasePath::default(),
///     module: PublicLocation::new("/start.js")?, stylesheet: None,
/// };
/// let _renderer = StreamLeptosRenderer::new(contract, assets, StreamLimits::default(), |page: &PageSnapshot<String>| {
///     Ok(view! { <p>{page.data.clone()}</p> })
/// })?;
/// # Ok::<(), stackpress_framework_view::ViewError>(())
/// ```
pub struct StreamLeptosRenderer<F> {
    /// Shared checked seed/HTML policy.
    contract: PageContract,
    /// Same checked external bootstrap/mount as buffered SSR.
    assets: PageAssets,
    /// Optional mounted external status asset; never executable inline content.
    status_script: Option<PublicLocation>,
    /// Explicit owned-chunk policy.
    policy: StreamLimits,
    /// Factory crosses the thread boundary; its local result never does.
    component: Arc<F>,
}
impl<F> StreamLeptosRenderer<F> {
    /// Validate a streamed renderer without starting a thread or evaluating a view.
    ///
    /// # Errors
    /// Rejects invalid assets/contracts or a chunk too small for safe recovery.
    pub fn new<T, V>(
        contract: PageContract,
        assets: PageAssets,
        policy: StreamLimits,
        component: F,
    ) -> Result<Self, ViewError>
    where
        F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
        V: IntoView,
    {
        assets.validate(&contract)?;
        if policy.render_timeout.is_zero()
            || std::time::Instant::now()
                .checked_add(policy.render_timeout)
                .is_none()
            || policy.max_chunk_bytes < FAILURE.len()
            || policy.max_chunk_bytes > contract.limits.html_bytes
        {
            return Err(ViewError::InvalidPolicy);
        }
        Ok(Self {
            contract,
            assets,
            status_script: None,
            policy,
            component: Arc::new(component),
        })
    }

    /// Opt into an early external status script without changing bootstrap ordering.
    ///
    /// The supplied application-relative asset is joined to the checked base.
    /// The host must serve [`STREAM_STATUS_SCRIPT`] there and explicitly allow its
    /// source in CSP. This does not add nonce support or relax an existing policy.
    /// Default renderers emit no status script and retain their existing assets.
    ///
    /// # Errors
    /// Rejects a joined location exceeding the public-location policy.
    pub fn with_status_script(mut self, script: PublicLocation) -> Result<Self, ViewError> {
        // Freeze the mounted location before starting any render or worker thread.
        self.status_script = Some(self.assets.base.join(script.as_str())?);
        Ok(self)
    }

    /// Stage owned Head/status before handing an exclusive Send body to the host.
    ///
    /// The host calls `HttpResponseExt::stream` with this body's mapped errors,
    /// applies `status`, and lets its one normal response lifecycle finish. A
    /// failed staging future yields an ordinary fresh-response error. Cancelling
    /// staging also joins the owned worker. Head is frozen before publication.
    ///
    /// # Errors
    /// Returns checked seed/policy/factory/Head errors before body publication.
    /// Factory/body-poll panic diagnostics are retained only as a ViewError source.
    /// A panic in Leptos's internal effect task is observed as deadline expiry:
    /// its discarded task handle is not joinable through stable public APIs.
    pub async fn render<T, V>(&self, snapshot: PageSnapshot<T>) -> Result<StreamPage, ViewError>
    where
        T: Serialize + Send + 'static,
        F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static,
        V: IntoView,
    {
        require_executor()?;
        snapshot.validate(self.contract.revision, &self.contract.build)?;
        if snapshot.locale != self.contract.locale || snapshot.theme != self.contract.theme {
            return Err(ViewError::ContractMismatch);
        }
        let seed = snapshot.script_json_with(self.contract.limits)?;
        let contract = self.contract.clone();
        let assets = self.assets.clone();
        let status_script = self.status_script.clone();
        let component = self.component.clone();
        let policy = self.policy;
        let status = snapshot.status.code();
        let input = RenderInput {
            snapshot,
            seed,
            contract,
            assets,
            status_script,
            component,
        };
        let body = stackpress_leptos::stream::StreamRenderer::render(
            render_limits(input.contract.limits),
            policy,
            FAILURE.into(),
            move |writer| async move { produce(input, writer).await },
        )
        .await
        .map_err(map_error)?;
        Ok(StreamPage {
            status,
            body: StreamBody { inner: body },
        })
    }
}

/// Precommitted page status and its exclusive bounded byte body.
pub struct StreamPage {
    /// Apply exactly once before host body publication.
    pub status: u16,
    /// Dropping the body closes capacity waits and joins the generic worker.
    pub body: StreamBody,
}
/// Page-error adapter over the generic body; adds no worker or buffering.
pub struct StreamBody {
    inner: stackpress_leptos::stream::StreamBody<ViewError>,
}
impl Stream for StreamBody {
    type Item = Result<Vec<u8>, ViewError>;
    /// Forward demand directly, mapping errors without changing frame bytes.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map(|item| item.map(|result| result.map_err(map_error)))
    }
}
struct RenderInput<T, F> {
    /// Checked public projection, never a live invocation.
    snapshot: PageSnapshot<T>,
    /// Bounded escaped public seed, prepared before thread admission.
    seed: String,
    /// Aggregate output and identity policy.
    contract: PageContract,
    /// Checked mounted external bootstrap assets.
    assets: PageAssets,
    /// Optional mounted status asset, independent of the completion-gated module.
    status_script: Option<PublicLocation>,
    /// Send/Sync factory whose produced view may remain local.
    component: Arc<F>,
}

/// Build only checked page framing in the generic worker's local Owner.
async fn produce<T, F, V>(
    input: RenderInput<T, F>,
    writer: stackpress_leptos::stream::StreamWriter<ViewError>,
) -> Result<(), stackpress_leptos::render::RenderError<ViewError>>
where
    T: Serialize,
    F: Fn(&PageSnapshot<T>) -> Result<V, ViewError>,
    V: IntoView,
{
    let RenderInput {
        snapshot,
        seed,
        contract: _,
        assets,
        status_script,
        component,
    } = input;
    let (meta, head) = ServerMetaContext::new();
    provide_context(meta);
    leptos_meta::provide_meta_context();
    super::routing::provide_route_context(assets.base.clone());
    provide_context(leptos_router::location::RequestUrl::new(&snapshot.location));
    let body = Box::pin(
        (component)(&snapshot)
            .map_err(stackpress_leptos::render::RenderError::Component)?
            .to_html_stream_in_order(),
    );
    let _ = view! { <leptos_meta::Title formatter=|value: String| escape_html(&value)/> }.to_html();
    if let Some(style) = &assets.stylesheet {
        let href = assets.base.join(style.as_str())?.as_str().to_owned();
        let _ = view! { <leptos_meta::Stylesheet id="stackpress-style" href/> }.to_html();
    }
    let module = assets.base.join(assets.module.as_str())?;
    let bootstrap = assets.script_json()?;
    // Async classic execution can update the notice while parsing is still pending.
    // Only checked external bytes execute; the ordinary module remains deferred.
    let status_script = status_script
        .map(|script| {
            format!(
                "<script async src=\"{}\"></script>",
                escape_html(script.as_str())
            )
        })
        .unwrap_or_default();
    // Saved explicit choices are complete SSR output; only unset admits the external sampler.
    let theme_head = if snapshot.theme.is_none() {
        crate::theme::bootstrap_tag(&assets.base, assets.theme_bootstrap.as_ref(), None)?
    } else {
        String::new()
    };
    // The unset marker permits one initial sample while the no-JS document remains readable Light.
    let theme_marker = if snapshot.theme.is_none() {
        " data-theme-unset"
    } else {
        ""
    };
    let opening = format!(
        "<!DOCTYPE html><html lang=\"{}\" data-theme=\"{}\"{theme_marker}><head>{theme_head}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><aside id=\"stackpress-stream-recovery\" role=\"status\">Loading or interrupted; reload if this page does not finish. <a href=\"{}\">Reload this page</a>.</aside>{status_script}<script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"application/json\" id=\"stackpress-assets\">{bootstrap}</script><script type=\"module\" src=\"{}\"></script><div id=\"app\">",
        snapshot.locale,
        snapshot.theme.as_deref().unwrap_or("light"),
        escape_html(&snapshot.location),
        escape_html(module.as_str())
    );
    writer
        .render(
            body,
            head,
            opening,
            |content, replay| {
                Ok(if replay.is_empty() {
                    content
                } else {
                    content + &format!("<template data-stackpress-replay>{replay}</template>")
                })
            },
            "</div><template id=\"stackpress-stream-complete\"></template></body></html>".into(),
        )
        .await
}
#[cfg(test)]
#[path = "stream_tests.rs"]
mod tests;
