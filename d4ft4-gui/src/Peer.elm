module Peer exposing (Mode(..), Model, Msg(..), addressString, init, statusString, update, view)

import Browser.Dom
import Html exposing (..)
import Html.Attributes
import Material.Icons exposing (mode)
import Maybe.Extra
import Task
import Theme
import W.Button as Button
import W.ButtonGroup as ButtonGroup
import W.Container as Container
import W.Heading as Heading
import W.InputInt as InputInt
import W.InputText as InputText
import W.Modal as Modal
import W.Text as Text


type Mode
    = Connect
    | Listen


modeLabel : Mode -> List (Html msg)
modeLabel mode =
    (case mode of
        Connect ->
            "Connect"

        Listen ->
            "Listen"
    )
        |> text
        |> List.singleton


modeAddressPlaceholder : Mode -> String
modeAddressPlaceholder mode =
    case mode of
        Connect ->
            "127.0.0.1"

        Listen ->
            "0.0.0.0"


type alias Model =
    { isOpen : Bool
    , mode : Mode
    , address : String
    , portNum : InputInt.Value
    }


addressOrDefault : Mode -> String -> Maybe String
addressOrDefault mode address =
    if String.isEmpty <| String.trim address then
        if String.isEmpty address then
            Just <| modeAddressPlaceholder mode

        else
            Nothing

    else
        Just address


addressString : Model -> Maybe String
addressString model =
    model.portNum
        |> InputInt.toInt
        |> Maybe.Extra.filter (\portNum -> portNum >= 0 && portNum <= 65535)
        |> Maybe.andThen (\portNum -> addressOrDefault model.mode model.address |> Maybe.map (Tuple.pair portNum))
        |> Maybe.map
            (\( portNum, address ) ->
                address
                    ++ ":"
                    ++ String.fromInt portNum
            )


statusString : Model -> Html msg
statusString model =
    case ( InputInt.toInt model.portNum, addressOrDefault model.mode model.address ) of
        ( Nothing, _ ) ->
            errorText "Invalid port"
        
        ( Just _, Nothing ) ->
            errorText "No address given"

        ( Just portNum, Just address ) ->
            if portNum < 0 then
                errorText "Port cannot be negative"

            else if portNum > 65535 then
                errorText "Max port number is 65535"

            else
                (case model.mode of
                    Connect ->
                        "Connect to "

                    Listen ->
                        "Listen, bind to "
                )
                    ++ address
                    ++ ":"
                    ++ String.fromInt portNum
                    |> text
                    |> List.singleton
                    |> pre []
                    |> List.singleton
                    |> Text.view [ Text.color Theme.baseForeground ]


init : Mode -> Model
init defaultMode =
    { isOpen = False
    , mode = defaultMode
    , address = ""
    , portNum = InputInt.init (Just 2581)
    }


view : Model -> Html Msg
view model =
    Modal.view []
        { isOpen = model.isOpen
        , onClose = Just Close
        , content =
            [ Container.view
                [ Container.vertical
                , Container.pad_4
                , Container.gap_3
                ]
                [ Heading.view [] [ text "Peer" ]
                , Container.view [ Container.alignLeft ]
                    [ ButtonGroup.view
                        [ ButtonGroup.small
                        , ButtonGroup.highlighted (\mode -> mode == model.mode)
                        ]
                        { items = [ Connect, Listen ]
                        , toLabel = modeLabel
                        , onClick = ModeChanged
                        }
                    ]
                , Container.view
                    [ Container.horizontal
                    , Container.gap_3
                    , Container.inline
                    ]
                    [ Container.view [ Container.fill ]
                        [ InputText.view
                            [ InputText.placeholder (modeAddressPlaceholder model.mode)
                            , InputText.small
                            , InputText.prefix [ text "Address" ]
                            , InputText.onEnter Close
                            , InputText.htmlAttrs [ Html.Attributes.id "peer-address-field" ]
                            ]
                            { onInput = AddressChanged, value = model.address }
                        ]
                    , Container.view [ Container.styleAttrs [ ( "flex-basis", "9em" ) ] ]
                        [ InputInt.view
                            [ InputInt.small
                            , InputInt.prefix [ text "Port" ]
                            , InputInt.onEnter Close
                            ]
                            { onInput = PortChanged, value = model.portNum }
                        ]
                    ]
                , Container.view [ Container.horizontal, Container.alignCenterY, Container.spaceBetween, Container.padTop_4 ]
                    [ statusString model
                    , Button.view [ Button.primary ] { label = [ text "Done" ], onClick = Close }
                    ]
                ]
            ]
        }


type Msg
    = ModeChanged Mode
    | AddressChanged String
    | PortChanged InputInt.Value
    | Open
    | Close
    | NoOp


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        ModeChanged mode ->
            ( { model | mode = mode }, Cmd.none )

        AddressChanged address ->
            ( { model | address = String.trim address }, Cmd.none )

        PortChanged portNum ->
            ( { model | portNum = portNum }, Cmd.none )

        Open ->
            ( { model | isOpen = True }, Browser.Dom.focus "peer-address-field" |> Task.attempt (\_ -> NoOp) )

        Close ->
            ( { model | isOpen = False }, Cmd.none )

        NoOp ->
            ( model, Cmd.none )


errorText : String -> Html msg
errorText t =
    Text.view [ Text.color Theme.dangerForeground ] [ text t ]
