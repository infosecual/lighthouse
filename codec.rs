
///! This handles the various supported encoding mechanism for the Eth 2.0 RPC. 
///!
///! Currently supported encodings are:
///! - ssz - Varint length-prefixed SSZ-encoding.


pub trait InnerCodec: Encoder + Decoder {
    type Error;

    pub fn decode_error(&mut self, &mut BytesMut) -> Result<Option<Self::Error>, <Self as Decoder>::Error>;
}

pub struct BaseInboundCodec<TCodec: InnerCodec> {
    /// Inner codec for handling various encodings
    inner: TCodec,
}

impl<TCodec> Encoder for BaseInboundCodec<TCodec>
where 
     TCodec: Encoder,
     <TCodec as Encoder>::Item = RPCResponse,
{
    type Item = RPCResponse;
    type Error = <TCodec as Encoder>::Error;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut BytesMut
    ) -> Result<(), Self::Error> {
        dst.clear();
        dst.reserve(1);
        dst.put_u8(item.as_u8);
        return self.inner.encode();
    }
}


impl<TCodec> Decoder for BaseInboundCodec<TCodec>
where 
   TCodec: Decoder,
   <TCodec as Decoder>::Item: RPCrequest,
   <TCodec as Decoder>::Error: From<RPCError>,
{

    type Item = RPCRequest;
    type Error = <TCodec as Decoder>::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut
    ) -> Result<Option<Self::Item>, Self::Error> {
        self.inner.decode(src)
    }
}

pub struct BaseOutboundCodec<TCodec> 
where
    TCodec: InnerCodec,
    <TCodec as Decoder>::Item = RPCResponse,
    <TCodec as InnerCodec>::ErrorItem = ErrorMessage,
    {
    /// Inner codec for handling various encodings
    inner: TCodec,
    /// Optimisation for decoding. True if the response code has been read and we are awaiting a
    /// response.
    response_code: Option<u8>,
}

impl<TCodec> Encoder for BaseOutboundCodec<TCodec>
where 
     TCodec: Encoder
{
    type Item = RPCRequest;
    type Error = <TCodec as Encoder>::Error;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut BytesMut
    ) -> Result<(), Self::Error> {
        self.inner.encode(item, dst)
    }
}


impl<TCodec> Decoder for BaseOutboundCodec<TCodec>
where 
   TCodec: InnerCodec,
   <TCodec as Decoder>::Error: From<RPCError>,
{

    type Item = RPCResponse;
    type Error = <TCodec as Decoder>::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut
    ) -> Result<Option<Self::Item>, Self::Error> {


        let response_code = {
            if let Some(resp_code) =  self.response_code {
                resp_code;
            }
            else {
                if src.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "no bytes received"));
                }
                let resp_byte = src.split_to(1);
                let resp_code_byte = [0; 1];
                resp_code_byte.copy_from_slice(&resp_byte);

                let resp_code = u8::from_be_bytes(resp_code_byte);

                if let Some(response) = RPCErrorResponse::internal_data(resp_code) {
                    self.response_code = None;
                    return response;
                }
                resp_code 
            }
        };

        if RPCErrorResponse::is_response(response_code) {
            // decode an actual response
            return self.inner.decode(src).map(|r| r.map(|resp| RPCErrorResponse::Success(resp)));
        }
        else {
            // decode an error
            return self.inner.decode_error(src).map(|r| r.map(|resp| RPCErrorResponse::from_error(response_code, resp)));
        }

    }

}



/// SSZ Input stream
pub struct SSZInboundSink<TSocket> {
    inner: 
    protocol: ProtocolId

impl<TSocket> for SSZInputStream<TSocket>
where
    TSocket: AsyncRead + AsyncWrite
{

    /// Set up the initial input stream object.
    pub fn new(incomming: TSocket, protocol: ProtocolId, max_size: usize) -> Self {

        // this type of stream should only apply to ssz protocols
        debug_assert!(protocol.encoding.as_str() == "ssz");

        let mut uvi_codec = UviBytes::default();
        uvi_codec.set_max_len(max_size);

        let inner = Framed::new(incomming, uvi_codec).from_err()
            .with(|response| {
              self.encode(response) 
            })
        .and_then(|bytes| {
            self.decode(request)
        }).into_future();

        //TODO: add timeout

        SSZInputStream {
            inner,
            protocol
        }
    }

    /// Decodes an SSZ-encoded RPCRequest.
    fn decode(&self, request: RPCRequest) {

        match self.protocol.message_name.as_str() {
            "hello" => match protocol.version.as_str() {
                "1.0.0" =>  Ok(RPCRequest::Hello(HelloMessage::from_ssz_bytes(&packet)?)),
                _ => Err(RPCError::InvalidProtocol("Unknown HELLO version")),
            },
            "goodbye" => match protocol.version.as_str() {
                "1.0.0" =>  Ok(RPCRequest::Goodbye(Goodbye::from_ssz_bytes(&packet)?)),
                _ => Err(RPCError::InvalidProtocol(
                    "Unknown GOODBYE version.as_str()",
                )),
            },
            "beacon_block_roots" => match protocol.version.as_str() {
                "1.0.0" => Ok(RPCRequest::BeaconBlockRoots(
                        BeaconBlockRootsRequest::from_ssz_bytes(&packet)?,
                    )),
                _ => Err(RPCError::InvalidProtocol(
                    "Unknown BEACON_BLOCK_ROOTS version.",
                )),
            },
            "beacon_block_headers" => match protocol.version.as_str() {
                "1.0.0" => Ok(RPCRequest::BeaconBlockHeaders(
                        BeaconBlockHeadersRequest::from_ssz_bytes(&packet)?,
                    )),
                _ => Err(RPCError::InvalidProtocol(
                    "Unknown BEACON_BLOCK_HEADERS version.",
                )),
            },
            "beacon_block_bodies" => match protocol.version.as_str() {
                "1.0.0" =>  Ok(RPCRequest::BeaconBlockBodies(
                        BeaconBlockBodiesRequest::from_ssz_bytes(&packet)?,
                    )),
                _ => Err(RPCError::InvalidProtocol(
                    "Unknown BEACON_BLOCK_BODIES version.",
                )),
            },
            "beacon_chain_state" => match protocol.version.as_str() {
                "1.0.0" =>  Ok(RPCRequest::BeaconChainState(
                        BeaconChainStateRequest::from_ssz_bytes(&packet)?,
                    )),
                _ => Err(RPCError::InvalidProtocol(
                    "Unknown BEACON_CHAIN_STATE version.",
                )),
            },
        }
    }

    fn encode(&self, response: RPCResponse) {

        // TODO: Add error code

        match response {
            RPCResponse::Hello(res) => res.as_ssz_bytes(),
            RPCResponse::Goodbye => unreachable!(),
            RPCResponse::BeaconBlockRoots(res) => res.as_ssz_bytes(),
            RPCResponse::BeaconBlockHeaders(res) => res.headers, // already raw bytes
            RPCResponse::BeaconBlockBodies(res) => res.block_bodies, // already raw bytes
            RPCResponse::BeaconChainState(res) => res.as_ssz_bytes(),
        }
    }

}

type SSZInboundOutput = stream::AndThen<sink::With<stream::FromErr<Framed<TSocket, UviBytes<Vec<u8>>>, RPCError>,
    RPCResponse,
    fn(RPCResponse) -> Result<Vec<u8>, RPCError>, 
    Result<Vec<u8>, RPCError>,
    >,
    fn(BytesMut) -> Result<RPCRequest, RPCError>,
    Result<RPCRequest, RPCError>
    >;

impl<TSocket> Sink for SSZInputStreamSink<TSocket> {

    type SinkItem = RPCResponse;
    type SinkError = RPCError;

    fn start_send(
    &mut self,
    item: Self::SinkItem
) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.inner.poll_complete()
    }
}

/* Outbound specific stream */

// Implement our own decoder to handle the response byte

struct SSZOutboundCodec



pub struct SSZOutboundStreamSink<TSocket> {
    inner: 
    protocol: ProtocolId

impl<TSocket> for SSZOutboundStreamSink<TSocket>
where
    TSocket: AsyncRead + AsyncWrite
{

    /// Set up the initial outbound stream object.
    pub fn new(socket: TSocket, protocol: ProtocolId, max_size: usize) -> Self {

        // this type of stream should only apply to ssz protocols
        debug_assert!(protocol.encoding.as_str() == "ssz");

        let mut uvi_codec = UviBytes::default();
        uvi_codec.set_max_len(max_size);

        let inner = Framed::new(socket, uvi_codec).from_err()
            .with(|request| {
              self.encode(request) 
            })
        .and_then(|bytes| {
            self.decode(response)
        });

        SSZOutboundStream {
            inner,
            protocol
        }
    }





    /// Decodes a response that was received on the same stream as a request. The response type should
    /// therefore match the request protocol type.
    pub fn decode(&self, response: Vec<u8>,
        protocol: ProtocolId,
        response_code: ResponseCode,
    ) -> Result<Self, RPCError> {
        match response_code {
            ResponseCode::EncodingError => Ok(RPCResponse::Error("Encoding error".into())),
            ResponseCode::InvalidRequest => {
                let response = match protocol.encoding.as_str() {
                    "ssz" => ErrorResponse::from_ssz_bytes(&packet)?,
                    _ => return Err(RPCError::InvalidProtocol("Unknown Encoding")),
                };
                Ok(RPCResponse::Error(format!(
                    "Invalid Request: {}",
                    response.error_message
                )))
            }
            ResponseCode::ServerError => {
                let response = match protocol.encoding.as_str() {
                    "ssz" => ErrorResponse::from_ssz_bytes(&packet)?,
                    _ => return Err(RPCError::InvalidProtocol("Unknown Encoding")),
                };
                Ok(RPCResponse::Error(format!(
                    "Remote Server Error: {}",
                    response.error_message
                )))
            }
            ResponseCode::Success => match protocol.message_name.as_str() {
                "hello" => match protocol.version.as_str() {
                    "1.0.0" => match protocol.encoding.as_str() {
                        "ssz" => Ok(RPCResponse::Hello(HelloMessage::from_ssz_bytes(&packet)?)),
                        _ => Err(RPCError::InvalidProtocol("Unknown HELLO encoding")),
                    },
                    _ => Err(RPCError::InvalidProtocol("Unknown HELLO version.")),
                },
                "goodbye" => Err(RPCError::Custom(
                    "GOODBYE should not have a response".into(),
                )),
                "beacon_block_roots" => match protocol.version.as_str() {
                    "1.0.0" => match protocol.encoding.as_str() {
                        "ssz" => Ok(RPCResponse::BeaconBlockRoots(
                            BeaconBlockRootsResponse::from_ssz_bytes(&packet)?,
                        )),
                        _ => Err(RPCError::InvalidProtocol(
                            "Unknown BEACON_BLOCK_ROOTS encoding",
                        )),
                    },
                    _ => Err(RPCError::InvalidProtocol(
                        "Unknown BEACON_BLOCK_ROOTS version.",
                    )),
                },
                "beacon_block_headers" => match protocol.version.as_str() {
                    "1.0.0" => match protocol.encoding.as_str() {
                        "ssz" => Ok(RPCResponse::BeaconBlockHeaders(
                            BeaconBlockHeadersResponse { headers: packet },
                        )),
                        _ => Err(RPCError::InvalidProtocol(
                            "Unknown BEACON_BLOCK_HEADERS encoding",
                        )),
                    },
                    _ => Err(RPCError::InvalidProtocol(
                        "Unknown BEACON_BLOCK_HEADERS version.",
                    )),
                },
                "beacon_block_bodies" => match protocol.version.as_str() {
                    "1.0.0" => match protocol.encoding.as_str() {
                        "ssz" => Ok(RPCResponse::BeaconBlockBodies(BeaconBlockBodiesResponse {
                            block_bodies: packet,
                        })),
                        _ => Err(RPCError::InvalidProtocol(
                            "Unknown BEACON_BLOCK_BODIES encoding",
                        )),
                    },
                    _ => Err(RPCError::InvalidProtocol(
                        "Unknown BEACON_BLOCK_BODIES version.",
                    )),
                },
                "beacon_chain_state" => match protocol.version.as_str() {
                    "1.0.0" => match protocol.encoding.as_str() {
                        "ssz" => Ok(RPCResponse::BeaconChainState(
                            BeaconChainStateResponse::from_ssz_bytes(&packet)?,
                        )),
                        _ => Err(RPCError::InvalidProtocol(
                            "Unknown BEACON_CHAIN_STATE encoding",
                        )),
                    },
                    _ => Err(RPCError::InvalidProtocol(
                        "Unknown BEACON_CHAIN_STATE version.",
                    )),
                },
            },
        }
    }

    fn encode(&self, response: RPCResponse) {

        match response {
            RPCResponse::Hello(res) => res.as_ssz_bytes(),
            RPCResponse::Goodbye => unreachable!(),
            RPCResponse::BeaconBlockRoots(res) => res.as_ssz_bytes(),
            RPCResponse::BeaconBlockHeaders(res) => res.headers, // already raw bytes
            RPCResponse::BeaconBlockBodies(res) => res.block_bodies, // already raw bytes
            RPCResponse::BeaconChainState(res) => res.as_ssz_bytes(),
        }
    }

}

type SSZOutboundStream = stream::AndThen<sink::With<stream::FromErr<Framed<TSocket, UviBytes<Vec<u8>>>, RPCError>,
    RPCResponse,
    fn(RPCResponse) -> Result<Vec<u8>, RPCError>, 
    Result<Vec<u8>, RPCError>,
    >,
    fn(BytesMut) -> Result<RPCRequest, RPCError>,
    Result<RPCRequest, RPCError>
    >;


impl<TSocket> Stream for SSZInputStreamSink<TSocket> {

    type Item = SSZInboundOutput;
    type Error = RPCError;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error>  {
        self.inner.poll()
    }
}

impl<TSocket> Sink for SSZInputStreamSink<TSocket> {

    type SinkItem = RPCResponse;
    type SinkError = RPCError;

    fn start_send(
    &mut self,
    item: Self::SinkItem
) -> Result<AsyncSink<Self::SinkItem>, Self::SinkError> {
        self.inner.start_send(item)
    }

    fn poll_complete(&mut self) -> Result<Async<()>, Self::SinkError> {
        self.inner.poll_complete()
    }
}







