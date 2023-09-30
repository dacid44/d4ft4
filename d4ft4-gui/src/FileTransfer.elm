module FileTransfer exposing (main)

import Browser
import Destination
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input


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
        [ layout []
            (Element.el [ width fill, height fill, Background.color (rgb255 49 51 56) ]
                (Element.column [ width fill, height fill, padding 16 ]
                    [ Element.row []
                        [ Input.text []
                            { onChange = IpAddressChanged
                            , text = model.ipAddress
                            , placeholder = Nothing
                            , label = Input.labelLeft [] (whiteText "IP Address: ")
                            }
                        ]
                    , Element.map DestinationMsg (Destination.view model.destination)
                    ]
                )
            )
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
whiteText : String -> Element msg
whiteText =
    Element.el [ Font.color (rgb255 255 255 255) ] << Element.text