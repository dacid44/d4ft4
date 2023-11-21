port module Messaging exposing (Call(..), Message, Response(..), callBackend, filesInList, receiveBackendMessage)

import Json.Decode as Decode exposing (Decoder)
import Json.Encode as Encode exposing (Value)


port callOpenFileDialog : Bool -> Cmd msg


port returnOpenFileDialog : (Maybe String -> msg) -> Sub msg



-- sends Message Call


port sendCall : Value -> Cmd msg



-- receives Message Response


port receiveResponse : (Value -> msg) -> Sub msg


type alias Message msg =
    { returnPath : List String
    , message : msg
    }


type Call
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
    = SetupComplete
    | TextSent
    | TextReceived String
    | FileSelected String
    | FilesSent
    | ReceivedFileList (List FileListItem)
    | ReceivedFiles
    | Error String


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
                                Decode.succeed SetupComplete

                            "TextSent" ->
                                Decode.succeed TextSent

                            "TextReceived" ->
                                Decode.field "content" <| Decode.map TextReceived Decode.string

                            "FileSelected" ->
                                Decode.field "content" <| Decode.map FileSelected Decode.string

                            "FilesSent" ->
                                Decode.succeed FilesSent

                            "ReceivedFileList" ->
                                Decode.field "content" <| Decode.map ReceivedFileList <| Decode.list decodeFileListItem

                            "ReceivedFiles" ->
                                Decode.succeed ReceivedFiles

                            "Error" ->
                                Decode.field "content" <| Decode.map Error Decode.string

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
