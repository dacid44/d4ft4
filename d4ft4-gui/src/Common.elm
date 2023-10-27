port module Common exposing (..)

import Home exposing (Msg)
import Json.Decode as Decode exposing (Decoder, decodeValue, field, map, oneOf, string)
import Json.Encode as Encode exposing (Value)


decodeResult : Decoder error -> Decoder value -> Decoder (Result error value)
decodeResult errDecoder valDecoder =
    oneOf
        [ Decode.map Err (field "Err" errDecoder)
        , Decode.map Ok (field "Ok" valDecoder)
        , Decode.fail "Result should be either 'Ok' or 'Err'"
        ]


decodeStringResult : Decoder (Result String String)
decodeStringResult =
    decodeResult string string


decodeIdentifiedStringResult : ( Int, Value ) -> ( Int, Maybe (Result String String) )
decodeIdentifiedStringResult ( conn_id, value ) =
    ( conn_id
    , decodeValue (decodeResult string string) value
        |> Result.toMaybe
    )


port callOpenFileDialog : Bool -> Cmd msg


port returnOpenFileDialog : (Maybe String -> msg) -> Sub msg



-- sends Message Call


port sendCall : Value -> Cmd msg



-- receives Message Response


port receiveResponse : (Value -> msg) -> Sub msg


type TransferMode
    = SendTextMode
    | ReceiveTextMode


type alias Message msg =
    { returnPath : List String
    , message : msg
    }


type Call
    = Setup { connId : Int, address : String, isServer : Bool, mode : TransferMode, password : String }
    | SendText { connId : Int, text : String }
    | ReceiveText { connId : Int }
    | ChooseFile
    | DropFiles { names : List String }


type Response
    = SetupComplete (Result String ())
    | TextSent (Result String ())
    | TextReceived (Result String String)
    | FileSelected (Result String String)


encodeCall : Message Call -> Value
encodeCall call =
    Encode.object
        [ ( "return-path", Encode.list Encode.string call.returnPath )
        , ( "message"
          , Encode.object
                (case call.message of
                    Setup { connId, address, isServer, mode, password } ->
                        [ ( "name", Encode.string "Setup" )
                        , ( "args"
                          , Encode.object
                                [ ( "conn-id", Encode.int connId )
                                , ( "address", Encode.string address )
                                , ( "is-server", Encode.bool isServer )
                                , ( "mode"
                                  , Encode.string
                                        (case mode of
                                            SendTextMode ->
                                                "send-text"

                                            ReceiveTextMode ->
                                                "receive-text"
                                        )
                                  )
                                , ( "password", Encode.string password )
                                ]
                          )
                        ]

                    SendText { connId, text } ->
                        [ ( "name", Encode.string "SendText" )
                        , ( "args"
                          , Encode.object
                                [ ( "conn-id", Encode.int connId )
                                , ( "text", Encode.string text )
                                ]
                          )
                        ]

                    ReceiveText { connId } ->
                        [ ( "name", Encode.string "ReceiveText" )
                        , ( "args"
                          , Encode.object [ ( "conn-id", Encode.int connId ) ]
                          )
                        ]

                    ChooseFile ->
                        [ ( "name", Encode.string "ChooseFile" ) ]

                    DropFiles { names } ->
                        [ ( "name", Encode.string "DropFiles" )
                        , ( "args"
                          , Encode.object [ ( "names", Encode.list Encode.string names ) ]
                          )
                        ]
                )
          )
        ]


decodeResponse : Decoder (Message Response)
decodeResponse =
    Decode.map2 Message
        (Decode.field "return-path" <| Decode.list Decode.string)
        (Decode.field "message"
            (Decode.field "name" Decode.string
                |> Decode.andThen
                    (\messageName ->
                        case messageName of
                            "SetupComplete" ->
                                Decode.map SetupComplete <| decodeResult Decode.string <| Decode.succeed ()

                            "TextSent" ->
                                Decode.map TextSent <| decodeResult Decode.string <| Decode.succeed ()

                            "TextReceived" ->
                                Decode.map TextReceived <| decodeResult Decode.string Decode.string

                            "FileSelected" ->
                                Decode.map FileSelected <| decodeResult Decode.string Decode.string

                            _ ->
                                Decode.fail "Unknown message name"
                    )
            )
        )
