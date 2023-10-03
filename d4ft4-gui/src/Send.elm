module Send exposing (Model, Msg(..), init, subscriptions, update, view)

import Common
import Html exposing (..)
import Html.Attributes exposing (style)
import Material.Icons as Filled
import Material.Icons.Types exposing (Coloring(..))
import Maybe.Extra
import Peer
import Theme
import W.Button as Button
import W.ButtonGroup as ButtonGroup
import W.Container as Container
import W.InputTextArea as InputTextArea
import W.InputText as InputText
import W.Modal as Modal
import W.Text as Text


type Mode
    = Text
    | File


modeLabel : Mode -> List (Html msg)
modeLabel mode =
    (case mode of
        Text ->
            "Send Text"

        File ->
            "Send Files"
    )
        |> text
        |> List.singleton


modeString : Mode -> String
modeString mode =
    case mode of
        Text ->
            "send-text"

        File ->
            "send-file"


type alias Model =
    { mode : Mode
    , text : String
    , files : List String
    , password : String
    , destination : Peer.Model
    , isSuccess : Bool
    , messages : List String
    }


init : Model
init =
    { mode = Text
    , text = ""
    , files = []
    , password = ""
    , destination = Peer.init Peer.Connect
    , isSuccess = False
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
            , Container.gap_3
            ]
            [ Button.view [ Button.icon ]
                { label = [ Filled.arrow_back 24 Inherit ]
                , onClick = backMsg
                }
            , Html.map convertMsg <|
                ButtonGroup.view [ ButtonGroup.highlighted (\mode -> mode == model.mode) ]
                    { items = [ Text, File ]
                    , toLabel = modeLabel
                    , onClick = ModeChanged
                    }
            ]
        , Html.map convertMsg <|
            case model.mode of
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

                File ->
                    text "file"
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
            Container.view
                [ Container.horizontal
                , Container.gap_3
                , Container.alignCenterY
                ]
                [ Button.view [] { label = [ text "Configure destination..." ], onClick = DestinationMsg Peer.Open }
                , Peer.statusString model.destination
                , if model.isSuccess then
                    Text.view [ Text.color Theme.primaryForeground ] [ text "Success!" ]

                  else
                    text ""
                , Html.map DestinationMsg (Peer.view model.destination)

                -- change this to an actual view later
                , Container.view [ Container.styleAttrs [ ( "margin-left", "auto" ) ] ]
                    [ Button.view [ Button.primary ] { label = [ text "Send" ], onClick = Send } ]
                ]
        ]


type Msg
    = ModeChanged Mode
    | TextChanged String
    | PasswordChanged String
    | DestinationMsg Peer.Msg
    | Send
    | ReturnSetup ( Int, Maybe String )
    | ReturnSendText ( Int, Maybe String )


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ModeChanged mode ->
            ( { model | mode = mode }, Cmd.none )

        TextChanged text ->
            ( { model | text = text }, Cmd.none )

        PasswordChanged password ->
            ( { model | password = password }, Cmd.none )

        DestinationMsg subMsg ->
            let
                ( subModel, subCmd ) =
                    Peer.update subMsg model.destination
            in
            ( { model | destination = subModel }, Cmd.map DestinationMsg subCmd )

        Send ->
            ( { model | isSuccess = False }
            , case Peer.addressString model.destination of
                Just address ->
                    Common.callSetup
                        { connId = 0
                        , address = address
                        , isServer = model.destination.mode == Peer.Listen
                        , mode = modeString model.mode
                        , password = ""
                        }

                Nothing ->
                    Cmd.none
            )

        ReturnSetup ( 0, message ) ->
            case message of
                Just error ->
                    ( { model | messages = model.messages ++ [ error ] }, Cmd.none )

                Nothing ->
                    ( model
                    , Common.callSendText
                        { connId = 0
                        , text = model.text
                        }
                    )

        ReturnSendText ( 0, message ) ->
            case message of
                Just error ->
                    ( { model | messages = model.messages ++ [ error ] }, Cmd.none )

                Nothing ->
                    ( { model | isSuccess = True }, Cmd.none )

        _ ->
            ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Common.returnSetup ReturnSetup
        , Common.returnSendText ReturnSendText
        ]
