use chainsight_cdk_macros::{chainsight_common, did_export, init_in, lens_method};
use ic_web3_rs::futures::{future::BoxFuture, FutureExt};
mod app;
chainsight_common!(60);
init_in!();
pub type HHIInput = u64;
pub type HHIOutput = u64;
lens_method!(HHIInput, HHIOutput);

did_export!("lens");
