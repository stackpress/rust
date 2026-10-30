"""One bounded outer proof, wrapping its nested Cargo once through the recorder."""
import pathlib,subprocess,sys,os
root=pathlib.Path.cwd()
result=subprocess.run(['cargo','run','-p','stackpress-framework-session','--features','native','--example','jwt_interop','--locked','--quiet'],check=True,text=True,capture_output=True,timeout=180)
# A disposable file retains token output outside the repository; no deployment secret is used.
path=pathlib.Path('/private/tmp/slice9a/rust-jwt.json');path.write_text(result.stdout)
subprocess.run(['node',str(root/'crates/framework/framework-session/proofs/interop.mjs'),sys.argv[1],str(path)],check=True,timeout=15)
