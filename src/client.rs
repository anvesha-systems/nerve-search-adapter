use std::io::{Write};
use std::os::unix::net::UnixStream;

use nerve_protocol::{MessageType, RequestId};
use tracing::{info, warn};

use nerve_protocol::io::FrameReader;

use crate::handler;
use crate::state::RequestState;

pub fn run(socket_path: &str)-> std::io::Result<()>{
    let mut stream = UnixStream::connect(socket_path)?;
    info!("connected to NERVE-CORE");

    let mut reader = FrameReader::new();
    let mut state = RequestState::new();

    loop{
        let frames = match reader.read_from(&mut stream){
            Ok(f) => f,
            Err(e) =>{
                warn!(error = %e, "protocol error, exiting");
                break;
            }
        };

        for frame in frames{
            match MessageType::try_from(frame.header.msg_type){
                Ok(MessageType::SearchQuery)=>{
                    if let Some(reply) = handler::handle_search(frame, &mut state){
                        stream.write_all(&reply)?;
                    }
                }
                Ok(MessageType::Cancel)=>{
                    state.cancel(RequestId(frame.header.request_id));
                }
                _ =>{
                    // ignore eveything else
                }
            }
        }
    }
    Ok(())
}