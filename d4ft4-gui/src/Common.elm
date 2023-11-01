port module Common exposing (Call(..), Message, Response(..), callBackend, filesInList, receiveBackendMessage)

import Home exposing (Msg)
import Json.Decode as Decode exposing (Decoder, decodeValue, field, map, oneOf, string)
import Json.Encode as Encode exposing (Value)
import Material.Icons exposing (password)


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


type
    Call
    -- = Setup { connId : Int, address : String, isServer : Bool, mode : TransferMode, password : String }
    = SetupSender SetupParams
    | SetupReceiver SetupParams
    | SendText { text : String }
    | ReceiveText
    | ChooseFile
    | DropFiles { names : List String }
    | SendFiles { names : List String }
    | ReceiveFileList
    | ReceiveFiles { allowlist : List String, outDir : Maybe String }


type alias SetupParams =
    { address : String
    , isServer : Bool
    , password : String
    }


type Response
    = SetupComplete (Result String ())
    | TextSent (Result String ())
    | TextReceived (Result String String)
    | FileSelected (Result String String)
    | FilesSent (Result String ())
    | ReceivedFileList (Result String (List FileListItem))
    | ReceivedFiles (Result String ())


type FileListItem
    = File { path : String, size : Int }
    | Directory { path : String }


filesInList : List FileListItem -> List { path : String, size : Int }
filesInList =
    List.filterMap
        (\item ->
            case item of
                File file ->
                    Just file

                Directory _ ->
                    Nothing
        )


encodeCall : Message Call -> Value
encodeCall call =
    Encode.object
        [ ( "return-path", Encode.list Encode.string call.returnPath )
        , ( "message"
          , Encode.object
                (case call.message of
                    -- Setup { connId, address, isServer, mode, password } ->
                    --     [ ( "name", Encode.string "Setup" )
                    --     , ( "args"
                    --       , Encode.object
                    --             [ ( "conn-id", Encode.int connId )
                    --             , ( "address", Encode.string address )
                    --             , ( "is-server", Encode.bool isServer )
                    --             , ( "mode"
                    --               , Encode.string
                    --                     (case mode of
                    --                         SendTextMode ->
                    --                             "send-text"
                    --
                    --                         ReceiveTextMode ->
                    --                             "receive-text"
                    --                     )
                    --               )
                    --             , ( "password", Encode.string password )
                    --             ]
                    --       )
                    --     ]
                    SetupSender setupParams ->
                        [ ( "name", Encode.string "SetupSender" )
                        , ( "args"
                          , encodeSetupParams setupParams
                          )
                        ]

                    SetupReceiver setupParams ->
                        [ ( "name", Encode.string "SetupReceiver" )
                        , ( "args"
                          , encodeSetupParams setupParams
                          )
                        ]

                    SendText { text } ->
                        [ ( "name", Encode.string "SendText" )
                        , ( "args"
                          , Encode.object [ ( "text", Encode.string text ) ]
                          )
                        ]

                    ReceiveText ->
                        [ ( "name", Encode.string "ReceiveText" ) ]

                    ChooseFile ->
                        [ ( "name", Encode.string "ChooseFile" ) ]

                    DropFiles { names } ->
                        [ ( "name", Encode.string "DropFiles" )
                        , ( "args"
                          , Encode.object [ ( "names", Encode.list Encode.string names ) ]
                          )
                        ]

                    SendFiles { names } ->
                        [ ( "name", Encode.string "SendFiles" )
                        , ( "args"
                          , Encode.object [ ( "names", Encode.list Encode.string names ) ]
                          )
                        ]

                    ReceiveFileList ->
                        [ ( "name", Encode.string "ReceiveFileList" ) ]

                    ReceiveFiles { allowlist, outDir } ->
                        [ ( "name", Encode.string "ReceiveFiles" )
                        , ( "args"
                          , Encode.object
                                [ ( "allowlist", Encode.list Encode.string allowlist )
                                , ( "out-dir", outDir |> Maybe.map Encode.string |> Maybe.withDefault Encode.null )
                                ]
                          )
                        ]
                )
          )
        ]


encodeSetupParams : SetupParams -> Value
encodeSetupParams { address, isServer, password } =
    Encode.object
        [ ( "address", Encode.string address )
        , ( "is-server", Encode.bool isServer )
        , ( "password", Encode.string password )
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

                            "FilesSent" ->
                                Decode.map FilesSent <| decodeResult Decode.string <| Decode.succeed ()

                            "ReceivedFileList" ->
                                Decode.map ReceivedFileList <| decodeResult Decode.string <| Decode.list decodeFileListItem

                            "ReceivedFiles" ->
                                Decode.map ReceivedFiles <| decodeResult Decode.string <| Decode.succeed ()

                            _ ->
                                Decode.fail "Unknown message name"
                    )
            )
        )


decodeFileListItem : Decoder FileListItem
decodeFileListItem =
    Decode.field "type" Decode.string
        |> Decode.andThen
            (\itemType ->
                case itemType of
                    "file" ->
                        Decode.map2 (\path size -> File { path = path, size = size })
                            (Decode.field "path" Decode.string)
                            (Decode.field "size" Decode.int)

                    "directory" ->
                        Decode.map (\path -> Directory { path = path }) (Decode.field "path" Decode.string)

                    _ ->
                        Decode.fail "Unknown file list item type"
            )


callBackend : Message Call -> Cmd msg
callBackend =
    encodeCall >> sendCall


receiveBackendMessage : (Result Decode.Error (Message Response) -> msg) -> Sub msg
receiveBackendMessage toMsg =
    receiveResponse (Decode.decodeValue decodeResponse >> toMsg)
