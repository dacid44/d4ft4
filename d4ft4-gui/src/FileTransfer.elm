module FileTransfer exposing (main)

import Browser
import Element exposing (..)
import Element.Background as Background
import Element.Border as Border
import Element.Input as Input

type alias Model =
    { ipAddress : String
    }

init : () -> (Model, Cmd Msg)
init _ = ( { ipAddress = "" }, Cmd.none )



view : Model -> Browser.Document Msg
view model =
    { title = "hello world"
    , body =
        [ layout []
            (Element.el [ width fill, height fill, Background.color (rgb255 49 51 56)]
                (Element.column [ width fill, height fill, padding 16 ]
                    [ Element.row []
                        [ Input.text [] { onChange = IpAddressChanged, text = model.ipAddress, placeholder = Nothing, label = Input.labelLeft [] (Element.text "IP Address: ") }
                        ]
                    ]
                )
            )
        ]
    }

type Msg
    = IpAddressChanged String

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        IpAddressChanged ipAddress -> ( { model | ipAddress = ipAddress }, Cmd.none )

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