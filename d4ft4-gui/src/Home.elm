module Home exposing (Model, Msg, init, update, view)

import Components
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


view : pMsg -> pMsg -> (Msg -> pMsg) -> (Html Msg -> Html pMsg) -> Model -> Html pMsg
view openSend openReceive convertMsg viewToolbar model =
    Container.view
        [ Container.vertical
        , Container.pad_4
        , Container.gap_3
        ]
        [ viewToolbar <| Heading.view [] [ text "Home" ]
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
