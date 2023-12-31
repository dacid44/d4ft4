module Receive exposing (Model, Msg(..), init, subscriptions, update, view)

import Components
import Filesize
import Html exposing (..)
import Html.Attributes exposing (name, selected, style)
import Material.Icons as Filled
import Material.Icons.Types exposing (Coloring(..))
import Messaging exposing (Call(..), Response(..))
import Peer
import Theme
import W.Button as Button
import W.ButtonGroup as ButtonGroup
import W.Container as Container
import W.DataRow as DataRow
import W.Divider as Divider
import W.InputCheckbox as InputCheckbox
import W.InputText as InputText
import W.InputTextArea as InputTextArea
import W.Text as Text


type Mode
    = Autodetect
    | Text
    | Files


modeLabel : Mode -> List (Html msg)
modeLabel mode =
    (case mode of
        Autodetect ->
            "Autodetect"

        Text ->
            "Receive Text"

        Files ->
            "Receive Files"
    )
        |> text
        |> List.singleton


modeString : Mode -> String
modeString mode =
    case mode of
        Autodetect ->
            "Autodetect"

        Text ->
            "Text"

        Files ->
            "Files"


type alias Model =
    { platform : String
    , mode : Mode
    , source : Peer.Model
    , text : String
    , password : String
    , files : List ReceivedFile
    , outDir : String
    , isConnected : Bool
    , messages : List String
    }


init : String -> Model
init platform =
    { platform = platform
    , mode = Text
    , source = Peer.init Peer.Listen
    , text = ""
    , password = ""
    , files = []
    , outDir = ""
    , isConnected = False
    , messages = []
    }


view : (Msg -> parentMsg) -> (Html Msg -> Html parentMsg) -> Model -> Html parentMsg
view convertMsg viewToolbar model =
    Container.view
        [ Container.vertical
        , Container.pad_4
        , Container.gap_3
        , Container.fill
        ]
        [ viewToolbar <|
            ButtonGroup.view
                [ ButtonGroup.disabled (\mode -> mode == Autodetect || (mode == Files && model.platform == "android"))
                , ButtonGroup.highlighted <| (==) model.mode
                ]
                { items = [ Autodetect, Text, Files ]
                , toLabel = modeLabel
                , onClick = ModeChanged
                }
        , Html.map convertMsg <|
            Container.view
                [ Container.horizontal
                , Container.gap_3
                , Container.alignCenterY
                ]
                [ Button.view [] { label = [ text "Configure source..." ], onClick = SourceMsg Peer.Open }
                , Peer.statusString model.source
                , Html.map SourceMsg (Peer.view model.source)
                ]
        , Html.map convertMsg <|
            Container.view
                [ Container.horizontal
                ]
                [ InputText.view
                    [ InputText.password
                    , InputText.small
                    , InputText.prefix [ Text.view [ Text.color Theme.baseForeground ] [ text "Password:" ] ]
                    ]
                    { onInput = PasswordChanged
                    , value = model.password
                    }
                ]
        , Html.map convertMsg <|
            -- change this to an actual view later
            Container.view
                [ Container.horizontal
                , Container.gap_3
                , Container.alignCenterY
                ]
                ([ Button.view [ Button.primary ] { label = [ text "Connect" ], onClick = Connect } ]
                    ++ (if model.isConnected then
                            [ Text.view [ Text.color Theme.primaryForeground ] [ text "Connected!" ]
                            , Button.view [ Button.primary ] { label = [ text "Receive text" ], onClick = ReceiveText }
                            ]

                        else
                            []
                       )
                )
        , Html.map convertMsg <|
            case model.mode of
                Autodetect ->
                    text "autodetect"

                Text ->
                    Container.view
                        [ Container.vertical
                        , Container.pad_4
                        , Container.gap_3
                        , Container.card
                        , Container.background Theme.neutralBackground
                        , Container.fill
                        ]
                        ([ Container.view
                            [ Container.card
                            , Container.background Theme.baseBackground
                            , Container.fill
                            ]
                            [ InputTextArea.view [ InputTextArea.htmlAttrs [ style "flex-grow" "1" ] ] { value = model.text, onInput = TextChanged }
                            ]
                         ]
                            ++ (model.messages |> List.map (text >> List.singleton >> pre []))
                        )

                Files ->
                    Container.view
                        [ Container.vertical
                        , Container.pad_4
                        , Container.gap_4
                        , Container.card
                        , Container.background Theme.neutralBackground
                        , Container.fill
                        ]
                        [ Container.view
                            [ Container.vertical
                            , Container.pad_2
                            , Container.gap_1
                            , Container.card
                            , Container.background Theme.baseBackground
                            , Container.fill
                            , Container.styleAttrs [ ( "height", "0px" ), ( "overflow-y", "auto" ) ]
                            ]
                            (model.files |> List.map viewReceivedFile |> List.intersperse (Divider.view [] []))
                        , Container.view
                            [ Container.horizontal
                            , Container.gap_3
                            , Container.fillSpace
                            ]
                            [ InputText.view [] { onInput = OutDirChanged, value = model.outDir }
                            , Button.view [ Button.primary ] { label = [ text "Receive selected files" ], onClick = ReceiveFiles }
                            ]
                        ]
        ]


type Msg
    = ModeChanged Mode
    | TextChanged String
    | PasswordChanged String
    | FileToggled String Bool
    | OutDirChanged String
    | SourceMsg Peer.Msg
    | Connect
    | ReceiveText
    | ReceiveFiles
    | ReceiveResponse (Messaging.Message Messaging.Response)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ModeChanged mode ->
            ( { model | mode = mode }, Cmd.none )

        TextChanged text ->
            ( { model | text = text }, Cmd.none )

        PasswordChanged password ->
            ( { model | password = password }, Cmd.none )

        FileToggled fileName selected ->
            ( { model
                | files =
                    model.files
                        |> List.map
                            (\file ->
                                if file.name == fileName then
                                    { file | selected = selected }

                                else
                                    file
                            )
              }
            , Cmd.none
            )

        OutDirChanged outDir ->
            ( { model | outDir = outDir }, Cmd.none )

        SourceMsg subMsg ->
            let
                ( subModel, subCmd ) =
                    Peer.update subMsg model.source
            in
            ( { model | source = subModel }, Cmd.map SourceMsg subCmd )

        Connect ->
            ( { model | isConnected = False }
            , case Peer.addressString model.source of
                Just address ->
                    Messaging.callBackend
                        { returnPath = [ "Receive", modeString model.mode ]
                        , message =
                            Messaging.SetupReceiver
                                { address = address
                                , isServer = model.source.mode == Peer.Listen
                                , password = model.password
                                }
                        }

                Nothing ->
                    Cmd.none
            )

        -- Maybe not needed anymore, unless maybe in autodetect?
        ReceiveText ->
            ( model
            , Messaging.callBackend
                { returnPath = [ "Receive" ]
                , message = Messaging.ReceiveText
                }
            )

        ReceiveFiles ->
            ( model
            , Messaging.callBackend
                { returnPath = [ "Receive" ]
                , message =
                    Messaging.ReceiveFiles
                        { allowlist =
                            model.files
                                |> List.filter .selected
                                |> List.map .name
                        , outDir =
                            if String.isEmpty model.outDir then
                                Nothing

                            else
                                Just model.outDir
                        }
                }
            )

        ReceiveResponse { returnPath, message } ->
            case ( returnPath, message ) of
                ( [ "Text" ], Messaging.SetupComplete ) ->
                    ( { model | isConnected = True }, Messaging.callBackend <| { returnPath = [ "Receive" ], message = Messaging.ReceiveText } )

                ( [ "Files" ], Messaging.SetupComplete ) ->
                    ( { model | isConnected = True }
                    , Messaging.callBackend
                        { returnPath = [ "Receive" ]
                        , message = Messaging.ReceiveFileList
                        }
                    )

                ( _, Messaging.TextReceived text ) ->
                    ( { model | text = text }, Cmd.none )

                ( _, Messaging.ReceivedFileList fileList ) ->
                    ( { model
                        | files =
                            fileList
                                |> Messaging.filesInList
                                |> List.map (\file -> initReceivedFile file.path file.size)
                      }
                    , Cmd.none
                    )

                ( _, Messaging.Error error ) ->
                    ( { model | messages = model.messages ++ [ error ] }, Cmd.none )

                _ ->
                    ( model, Cmd.none )



-- _ ->
--     ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- File list items


type alias ReceivedFile =
    { name : String
    , size : Int
    , selected : Bool
    }


initReceivedFile : String -> Int -> ReceivedFile
initReceivedFile name size =
    { name = name, size = size, selected = False }


viewReceivedFile : ReceivedFile -> Html Msg
viewReceivedFile file =
    DataRow.viewNextExtra
        [ DataRow.padding 0 ]
        { left = [ InputCheckbox.view [] { value = file.selected, onInput = FileToggled file.name } ]
        , header = []
        , main = [ text file.name ]
        , footer = [ text <| Filesize.format file.size ]
        , right = []
        }
