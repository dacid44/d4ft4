port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (value, placeholder)
import Html.Events exposing (onInput)
import Html.Events exposing (onClick)


port callAdd : { a : Int, b : Int } -> Cmd msg


port returnAdd : (Int -> msg) -> Sub msg


type alias Model =
    { a : String
    , b : String
    , result : Maybe Int
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model "" "" Nothing, Cmd.none )


view : Model -> Browser.Document Msg
view model =
    { title = "D4FT4"
    , body =
        [ input [ placeholder "a", value model.a, onInput AChanged ] []
        , input [ placeholder "b", value model.b, onInput BChanged ] []
        , button [ onClick Calculate ] [ text "calculate" ]
        , model.result |> Maybe.map (String.fromInt >> text) |> Maybe.withDefault (text "no answer")
        ]
    }


type Msg
    = AChanged String
    | BChanged String
    | Calculate
    | ReturnAdd Int


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =  
    case msg of
        AChanged a ->
            ( { model | a = a }, Cmd.none )

        BChanged b ->
            ( { model | b = b }, Cmd.none )

        Calculate ->
            ( model
            , case ( String.toInt model.a, String.toInt model.b ) of
                ( Just aInt, Just bInt ) ->
                    callAdd { a = aInt, b = bInt }

                _ ->
                    Cmd.none
            )

        ReturnAdd result ->
            ( { model | result = Just result }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    returnAdd ReturnAdd


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
