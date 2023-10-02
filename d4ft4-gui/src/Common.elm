port module Common exposing (..)

port callSetup : { connId : Int, address : String, isServer : Bool, mode : String, password : String } -> Cmd msg
port returnSetup : (( Int, Maybe String ) -> msg) -> Sub msg

port callSendText : { connId : Int, text : String } -> Cmd msg
port returnSendText : (( Int, Maybe String ) -> msg) -> Sub msg

port callReceiveText : { connId : Int } -> Cmd msg
port returnReceiveText : (( Int, String ) -> msg) -> Sub msg

port callSendFile : { connId : Int, path : String } -> Cmd msg
port returnSendFile : (( Int, Maybe String ) -> msg) -> Sub msg

port callReceiveFile : { connId : Int, path : String } -> Cmd msg
port returnReceiveFile : (( Int, Maybe String ) -> msg) -> Sub msg
