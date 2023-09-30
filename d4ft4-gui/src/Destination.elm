module Destination exposing (Mode, Model, Msg, init, update, view)

import Html exposing (..)
import Html.Attributes exposing (style, width)
import W.Button as Button
import W.Container as Container
import W.Heading as Heading
import W.InputInt as InputInt
import W.InputSelect as InputSelect
import W.InputText as InputText
import W.Modal as Modal


type Mode
    = Connect
    | Listen


modeSelectMessage : Mode -> String
modeSelectMessage mode =
    case mode of
        Connect ->
            "Connect"

        Listen ->
            "Listen"


type alias Model =
    { mode : Mode
    , address : String
    , portNum : InputInt.Value
    }


init : Model
init =
    { mode = Connect
    , address = ""
    , portNum = InputInt.init (Just 2581)
    }


view : String -> Model -> Html Msg
view id model =
    Modal.viewToggableWithAutoClose []
        { id = id
        , content =
            [ Container.view [ Container.vertical, Container.pad_4, Container.gap_3 ]
                [ Heading.view [] [ text "Destination" ]
                , InputSelect.view [ InputSelect.small ]
                    { value = model.mode
                    , options = [ Connect, Listen ]
                    , toLabel = modeSelectMessage
                    , onInput = ModeChanged
                    }
                , Container.view
                    [ Container.horizontal
                    , Container.gap_3
                    , Container.inline
                    ]
                    [ Container.view [ Container.styleAttrs [ ( "flex", "2 2 auto" ) ] ]
                        [ InputText.view
                            [ InputText.placeholder "191.168.1.10"
                            , InputText.small
                            , InputText.prefix [ text "Address" ]
                            ]
                            { onInput = AddressChanged, value = model.address }
                        ]
                    , Container.view [ Container.styleAttrs [ ( "flex-basis", "9em" ) ] ]
                        [ InputInt.view
                            [ InputInt.small, InputInt.prefix [ text "Port" ] ]
                            { onInput = PortChanged, value = model.portNum }
                        ]
                    ]
                , Container.view [ Container.horizontal, Container.alignRight ]
                    [ Modal.viewToggle id
                        [ Button.viewDummy [] [ text "Done" ] ]
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
            ( { model | address = address }, Cmd.none )

        PortChanged portNum ->
            ( { model | portNum = portNum }, Cmd.none )
