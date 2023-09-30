module Destination exposing (Mode, Model, init, view, Msg, update)

import Element exposing (..)
import Element.Input as Input
import Dropdown


type Mode
    = Connect
    | Listen

modeMessage : Mode -> Element Msg
modeMessage mode =
    case mode of
        Connect -> text "Connect to:"
        Listen -> text "Listen at:"


modeSelectMessage : Bool -> Bool -> Mode -> Element Msg
modeSelectMessage _ _ mode =
    case mode of
        Connect -> text "Connect"
        Listen -> text "Listen"

modeAddressPlaceholder : Mode -> Input.Placeholder msg
modeAddressPlaceholder mode =
    Input.placeholder []
        (case mode of
            Connect -> text "192.168.1.10"
            Listen -> text "0.0.0.0"
        )


type alias Model =
    { mode : Mode
    , address : String
    , portNum : Int
    , dropdownState : Dropdown.State Mode
    }

init : Model
init =
    { mode = Connect
    , address = ""
    , portNum = 2581
    , dropdownState = Dropdown.init "destination-mode-dropdown"
    }

view : Model -> Element Msg
view model =
    column [ padding 8, spacing 8 ]
        [ Dropdown.view dropdownConfig model.mode model.dropdownState
        , row [ spacing 8 ]
            [ Input.text []
                { onChange = AddressChanged
                , text = model.address
                , placeholder = Just (modeAddressPlaceholder model.mode)
                , label = Input.labelLeft [] (text "Address:")
                }
            , Input.text []
                { onChange = PortChanged
                , text = String.fromInt model.portNum
                , placeholder = Nothing
                , label = Input.labelLeft [] (text "Port:")
                }
            ]
        ]

dropdownConfig : Dropdown.Config Mode Msg Mode
dropdownConfig =
    Dropdown.basic
        { itemsFromModel = (\_ -> [ Connect, Listen ])
        , selectionFromModel = Just
        , dropdownMsg = DropdownMsg
        , onSelectMsg = ModeChanged
        , itemToPrompt = modeMessage
        , itemToElement = modeSelectMessage
        }

type Msg
    = DropdownMsg (Dropdown.Msg Mode)
    | ModeChanged (Maybe Mode)
    | AddressChanged String
    | PortChanged String

update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        DropdownMsg subMsg ->
            let ( state, cmd ) = Dropdown.update dropdownConfig subMsg model.mode model.dropdownState in
                ( { model | dropdownState = state }, cmd )

        ModeChanged (Just mode) ->
            ( { model | mode = mode }, Cmd.none )

        ModeChanged Nothing ->
            ( model, Cmd.none )

        AddressChanged address ->
            ( { model | address = address }, Cmd.none )

        PortChanged portNum ->
            ( { model | portNum = String.toInt portNum |> Maybe.withDefault model.portNum }, Cmd.none )

