module Receive exposing (Model, Msg(..), init, subscriptions, update, view)

import Common
import Html exposing (..)
import Html.Attributes exposing (style)
import Material.Icons as Filled
import Material.Icons.Types exposing (Coloring(..))
import Peer
import Theme
import W.Button as Button
import W.ButtonGroup as ButtonGroup
import W.Container as Container
import W.InputText as InputText
import W.InputTextArea as InputTextArea
import W.Text as Text


type Mode
    = Autodetect
    | Text
    | File


modeLabel : Mode -> List (Html msg)
modeLabel mode =
    (case mode of
        Autodetect ->
            "Autodetect"

        Text ->
            "Receive Text"

        File ->
            "Receive Files"
    )
        |> text
        |> List.singleton


modeString : Mode -> String
modeString mode =
    case mode of
        Autodetect ->
            "receive-autodetect"

        Text ->
            "receive-text"

        File ->
            "receive-file"


type alias Model =
    { mode : Mode
    , source : Peer.Model
    , text : String
    , password : String
    , isConnected : Bool
    , messages : List String
    }


init : Model
init =
    { mode = Text
    , source = Peer.init Peer.Listen
    , text = ""
    , password = ""
    , isConnected = False
    , messages = []
    }


view : pMsg -> (Msg -> pMsg) -> Model -> Html pMsg
view backMsg convertMsg model =
    Container.view
        [ Container.vertical
        , Container.pad_4
        , Container.gap_3
        , Container.fill
        ]
        [ Container.view
            [ Container.horizontal
            , Container.padBottom_4
            , Container.gap_3
            ]
            [ Button.view [ Button.icon ]
                { label = [ Filled.arrow_back 24 Inherit ]
                , onClick = backMsg
                }
            , Html.map convertMsg <|
                ButtonGroup.view
                    [ ButtonGroup.disabled (\mode -> mode == Autodetect)
                    , ButtonGroup.highlighted (\mode -> mode == model.mode)
                    ]
                    { items = [ Autodetect, Text, File ]
                    , toLabel = modeLabel
                    , onClick = ModeChanged
                    }
            ]
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
                            , Button.view [ Button.primary ] { label = [ text "Receive text" ], onClick = Receive }
                            ]

                        else
                            []
                       )
                )
        , Html.map convertMsg <|
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
        ]


type Msg
    = ModeChanged Mode
    | TextChanged String
    | PasswordChanged String
    | SourceMsg Peer.Msg
    | Connect
    | Receive
    | ReturnSetup ( Int, Maybe String )
    | ReturnReceiveText ( Int, Maybe (Result String String) )
    | ReceiveResponse (Common.Message Common.Response)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ModeChanged mode ->
            ( { model | mode = mode }, Cmd.none )

        TextChanged text ->
            ( { model | text = text }, Cmd.none )

        PasswordChanged password ->
            ( { model | password = password }, Cmd.none )

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
                    Common.sendCall <|
                        Common.encodeCall
                            { returnPath = [ "Receive", "Text" ]
                            , message =
                                Common.Setup
                                    { connId = 1
                                    , address = address
                                    , isServer = model.source.mode == Peer.Listen
                                    , mode = Common.ReceiveTextMode
                                    , password = model.password
                                    }
                            }

                Nothing ->
                    Cmd.none
            )

        Receive ->
            ( model
            , Common.sendCall <|
                Common.encodeCall
                    { returnPath = [ "Receive" ]
                    , message = Common.ReceiveText { connId = 1 }
                    }
            )

        ReceiveResponse { returnPath, message } ->
            case ( returnPath, message ) of
                ( _, Common.SetupComplete (Err error) ) ->
                    ( { model | messages = model.messages ++ [ error ] }, Cmd.none )

                ( [ "Text" ], Common.SetupComplete (Ok _) ) ->
                    ( { model | isConnected = True }, Cmd.none )

                ( _, Common.TextReceived (Err error) ) ->
                    ( { model | messages = model.messages ++ [ error ] }, Cmd.none )

                ( _, Common.TextReceived (Ok text) ) ->
                    ( { model | text = text }, Cmd.none )

                _ ->
                    ( model, Cmd.none )

        _ ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none
