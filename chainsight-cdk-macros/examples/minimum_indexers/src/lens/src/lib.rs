pub type HHIInput = u64;
pub type HHIOutput = u64;
use chainsight_cdk_macros::{chainsight_common, did_export, init_in, lens_method};
use ic_web3_rs::futures::{future::BoxFuture, FutureExt};
mod app;
lens_method!(HHIInput, HHIOutput);

chainsight_common!(60);
init_in!();

did_export!("lens");
