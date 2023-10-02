module Home exposing (Model, Msg, init, update, view)

import Html exposing (..)
import W.Button as Button
import W.Container as Container
import W.Heading as Heading


type alias Model =
    { history : ()
    }


init : Model
init =
    { history = ()
    }


view : pMsg -> pMsg -> (Msg -> pMsg) -> Model -> Html pMsg
view openSend openReceive convertMsg model =
    Container.view
        [ Container.vertical
        , Container.pad_4
        , Container.gap_3
        ]
        [ Heading.view [] [ text "Home" ]
        , Container.view
            [ Container.horizontal
            , Container.gap_3
            ]
            [ Button.view [] { label = [ text "Send something" ], onClick = openSend }
            , Button.view [] { label = [ text "Receive something" ], onClick = openReceive }
            ]
        ]


type Msg
    = None


update : Msg -> Model -> ( Model, Cmd Msg )
update _ model =
    ( model, Cmd.none )
