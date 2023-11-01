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
import W.DataRow as DataRow
import W.Divider as Divider
import W.InputCheckbox as InputCheckbox
import W.InputText as InputText
import W.InputTextArea as InputTextArea
import W.Text as Text


type Mode
    = Text
    | Files


modeLabel : Mode -> List (Html msg)
modeLabel mode =
    (case mode of
        Text ->
            "Send Text"

        Files ->
            "Send Files"
    )
        |> text
        |> List.singleton


modeString : Mode -> String
modeString mode =
    case mode of
        Text ->
            "Text"

        Files ->
            "Files"


type alias Model =
    { mode : Mode
    , text : String
    , files : List LoadedFile
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


view : parentMsg -> (Msg -> parentMsg) -> Model -> Html parentMsg
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
                    { items = [ Text, Files ]
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
                            (model.files |> List.map viewLoadedFile |> List.intersperse (Divider.view [] []))
                        , Container.view
                            [ Container.horizontal
                            , Container.gap_3
                            , Container.fillSpace
                            ]
                            [ Button.view [ Button.primary ] { label = [ text "Pick File" ], onClick = SelectFile }
                            , Button.view [ Button.danger ] { label = [ text "Delete" ], onClick = DeleteSelectedFiles }
                            ]
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
                , Container.view [ Container.fill ] []
                , Button.view [ Button.primary ] { label = [ text "Send" ], onClick = Send }
                ]
        ]


type Msg
    = ModeChanged Mode
    | TextChanged String
    | PasswordChanged String
    | FileToggled String Bool
    | DeleteSelectedFiles
    | DestinationMsg Peer.Msg
    | Send
    | ReceiveResponse (Common.Message Common.Response)
    | SelectFile
    | PathAdded (Maybe String)


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

        DeleteSelectedFiles ->
            ( { model | files = model.files |> List.filter (not << .selected) }
            , Common.callBackend
                { returnPath = [ "Send" ]
                , message = Common.DropFiles { names = model.files |> List.filter .selected |> List.map .name }
                }
            )

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
                    Common.callBackend
                        { returnPath = [ "Send", modeString model.mode ]
                        , message =
                            Common.SetupSender
                                { address = address
                                , isServer = model.destination.mode == Peer.Listen
                                , password = model.password
                                }
                        }

                Nothing ->
                    Cmd.none
            )

        ReceiveResponse { returnPath, message } ->
            case ( returnPath, message ) of
                ( [ "Text" ], Common.SetupComplete ) ->
                    ( model
                    , Common.callBackend
                        { returnPath = [ "Send" ]
                        , message = Common.SendText { text = model.text }
                        }
                    )

                ( [ "Files" ], Common.SetupComplete ) ->
                    ( model
                    , Common.callBackend
                        { returnPath = [ "Send" ]
                        , message = Common.SendFiles { names = model.files |> List.filter .selected |> List.map .name }
                        }
                    )

                ( _, Common.TextSent ) ->
                    ( { model | isSuccess = True }, Cmd.none )

                ( _, Common.FileSelected name ) ->
                    ( { model | files = model.files ++ [ initLoadedFile name ] }, Cmd.none )

                ( _, Common.Error error ) ->
                    ( { model | messages = model.messages ++ [ error ] }, Cmd.none )

                _ ->
                    ( model, Cmd.none )

        SelectFile ->
            ( model
            , Common.callBackend
                { returnPath = [ "Send" ]
                , message = Common.ChooseFile
                }
            )

        PathAdded path ->
            ( { model | files = model.files ++ (Maybe.Extra.toList path |> List.map initLoadedFile) }, Cmd.none )



-- _ ->
--     ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-- File list items


type alias LoadedFile =
    { name : String
    , selected : Bool
    }


initLoadedFile : String -> LoadedFile
initLoadedFile name =
    { name = name
    , selected = False
    }


viewLoadedFile : LoadedFile -> Html Msg
viewLoadedFile file =
    DataRow.viewNext
        [ DataRow.onClick <| FileToggled file.name <| not file.selected
        , DataRow.padding 0
        ]
        { left = [ InputCheckbox.view [] { value = file.selected, onInput = FileToggled file.name } ]
        , main = [ text file.name ]
        , right = []
        }
