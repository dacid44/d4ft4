port module Common exposing (..)

import Json.Decode exposing (Decoder, oneOf, map, field, string, decodeValue)
import Json.Encode exposing (Value)

port callSetup : { connId : Int, address : String, isServer : Bool, mode : String, password : String } -> Cmd msg
port returnSetup : (( Int, Maybe String ) -> msg) -> Sub msg

port callSendText : { connId : Int, text : String } -> Cmd msg
port returnSendText : (( Int, Maybe String ) -> msg) -> Sub msg

port callReceiveText : { connId : Int } -> Cmd msg

-- Returns an ( Int, Result String String )
port returnReceiveText : (( Int, Json.Encode.Value ) -> msg) -> Sub msg

port callSendFile : { connId : Int, path : String } -> Cmd msg
port returnSendFile : (( Int, Maybe String ) -> msg) -> Sub msg

port callReceiveFile : { connId : Int, path : String } -> Cmd msg
port returnReceiveFile : (( Int, Maybe String ) -> msg) -> Sub msg


decodeResult : Decoder value -> Decoder error -> Decoder (Result error value)
decodeResult valDecoder errDecoder =
    oneOf
        [ map Ok (field "Ok" valDecoder)
        , map Err (field "Err" errDecoder)
        ]

decodeStringResult : Decoder (Result String String)
decodeStringResult =
    decodeResult string string

decodeIdentifiedStringResult : ( Int, Json.Encode.Value ) -> ( Int, Maybe (Result String String) )
decodeIdentifiedStringResult ( conn_id, value ) =
    ( conn_id
    , decodeValue (decodeResult string string) value
        |> Result.toMaybe
    )