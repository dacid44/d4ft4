module FileTransfer exposing (main)

import Browser
import Destination
import Html exposing (..)
import W.Styles
import W.Modal as Modal
import W.Button as Button
import W.Container as Container exposing (..)
import Theme


type alias Model =
    { ipAddress : String
    , destination : Destination.Model
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { ipAddress = "", destination = Destination.init }, Cmd.none )


view : Model -> Browser.Document Msg
view model =
    { title = "hello world"
    , body =
        [ W.Styles.globalStyles
        , Theme.globalProviderWithDarkMode
            { light = Theme.lightTheme
            , dark = Theme.darkTheme
            , strategy = Theme.systemStrategy
            }
        , Modal.viewToggle "main-destination"
            [ Button.viewDummy [] [ text "Modal toggle" ] ]
        , Html.map DestinationMsg (Destination.view "main-destination" model.destination)
        ]
    }


type Msg
    = IpAddressChanged String
    | DestinationMsg Destination.Msg


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        IpAddressChanged ipAddress ->
            ( { model | ipAddress = ipAddress }, Cmd.none )

        DestinationMsg subMsg ->
            let ( subModel, subCmd ) = Destination.update subMsg model.destination in
                ( { model | destination = subModel }, Cmd.map DestinationMsg subCmd )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


-- utility functions
--whiteText : String -> Element msg
--whiteText =
--    Element.el [ Font.color (rgb255 255 255 255) ] << Element.text