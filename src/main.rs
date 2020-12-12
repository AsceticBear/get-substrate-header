use jsonrpsee::raw::RawClient;
use jsonrpsee::transport::ws::WsTransportClient;
use jsonrpsee::Client;
use sp_core::Decode;
use sp_finality_grandpa::{ConsensusLog, GRANDPA_ENGINE_ID, ScheduledChange};
use sp_runtime::{generic, traits::BlakeTwo256};

pub type BlockNumber = u32;
pub type Hasher = BlakeTwo256;
pub type Header = generic::Header<BlockNumber, Hasher>;

jsonrpsee::rpc_api! {
    pub(crate) Substrate {
        #[rpc(method = "chain_getHeader", positional_params)]
        fn chain_get_header(block_hash: String) -> Header;
    }
}

fn main() {
    async_std::task::block_on(async {
        let url = "wss://cc1.darwinia.network";
        let transport = WsTransportClient::new(&url).await.unwrap();
        let raw_client = RawClient::new(transport);
        let client: Client = raw_client.into();

        let block_hash = "0x35315dd6df1ba6a58e79a2c40a2a3419ae6634c9738f573e3cc941e3a6d852fb";

        let header = Substrate::chain_get_header(&client, block_hash)
            .await
            .unwrap();

        let res: Option<ScheduledChange<BlockNumber>> = sp_runtime::traits::Header::digest(&header)
            .log(|log| {
                log.as_consensus().and_then(|(engine_id, log)| {
                    if engine_id == GRANDPA_ENGINE_ID {
                        Some(log)
                    } else {
                        None
                    }
                })
            })
            .and_then(|log| ConsensusLog::<BlockNumber>::decode(&mut &log[..]).ok())
            .and_then(|log| match log {
                ConsensusLog::ScheduledChange(scheduled_change) => Some(scheduled_change),
                _ => None,
            });
        println!("Great, got the consensus log {:?}", res);
    });
}
