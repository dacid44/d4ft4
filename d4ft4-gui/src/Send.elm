module Send exposing (view)

import Element exposing (..)


type alias Model =
    { text : String
    , files : List String
    }

view : Model -> Element Msg
view model =
    row

type Msg =
    TextChanged String