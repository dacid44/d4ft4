module Peer exposing (Mode(..), Model, Msg, addressString, init, statusString, update, view)

import Html exposing (..)
import Html.Attributes exposing (style, width)
import Maybe.Extra
import Theme
import W.Button as Button
import W.ButtonGroup as ButtonGroup
import W.Container as Container
import W.Heading as Heading
import W.InputInt as InputInt
import W.InputSelect as InputSelect
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
            "192.168.1.10"

        Listen ->
            "0.0.0.0"


type alias Model =
    { mode : Mode
    , address : String
    , portNum : InputInt.Value
    }


addressString : Model -> Maybe String
addressString model =
    model.portNum
        |> InputInt.toInt
        |> Maybe.Extra.filter (\portNum -> portNum >= 0 && portNum <= 65535 && not (String.isEmpty model.address))
        |> Maybe.map (\portNum -> model.address ++ ":" ++ String.fromInt portNum)


statusString : Model -> Html msg
statusString model =
    case InputInt.toInt model.portNum of
        Nothing ->
            errorText "Invalid port"

        Just portNum ->
            if String.isEmpty model.address then
                errorText "No address given"

            else if portNum < 0 then
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
                    ++ model.address
                    ++ ":"
                    ++ String.fromInt portNum
                    |> text
                    |> List.singleton
                    |> pre []
                    |> List.singleton
                    |> Text.view [ Text.color Theme.baseForeground ]


init : Mode -> Model
init defaultMode =
    { mode = defaultMode
    , address = ""
    , portNum = InputInt.init (Just 2581)
    }


view : String -> Model -> Html Msg
view id model =
    Modal.viewToggableWithAutoClose []
        { id = id
        , content =
            [ Container.view
                [ Container.vertical
                , Container.pad_4
                , Container.gap_3
                ]
                [ Heading.view [] [ text "Peer" ]
                , Container.view [ Container.styleAttrs [ ( "align-items", "start" ) ] ]
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
                            ]
                            { onInput = AddressChanged, value = model.address }
                        ]
                    , Container.view [ Container.styleAttrs [ ( "flex-basis", "9em" ) ] ]
                        [ InputInt.view
                            [ InputInt.small
                            , InputInt.prefix [ text "Port" ]
                            ]
                            { onInput = PortChanged, value = model.portNum }
                        ]
                    ]
                , Container.view [ Container.horizontal, Container.alignCenterY, Container.spaceBetween, Container.padTop_4 ]
                    [ statusString model
                    , Modal.viewToggle id
                        [ Button.viewDummy [ Button.primary ] [ text "Done" ] ]
                    ]
                ]
            ]
        }


type Msg
    = ModeChanged Mode
    | AddressChanged String
    | PortChanged InputInt.Value


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        --DropdownMsg subMsg ->
        --    let ( state, cmd ) = Dropdown.update dropdownConfig subMsg model.mode model.dropdownState in
        --        ( { model | dropdownState = state }, cmd )
        ModeChanged mode ->
            ( { model | mode = mode }, Cmd.none )

        AddressChanged address ->
            ( { model | address = String.trim address }, Cmd.none )

        PortChanged portNum ->
            ( { model | portNum = portNum }, Cmd.none )


errorText : String -> Html msg
errorText t =
    Text.view [ Text.color Theme.dangerForeground ] [ text t ]
