port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (placeholder, value)
import Html.Events exposing (onClick, onInput)
import Html.Attributes exposing (type_)


port callAdd : { a : Int, b : Int } -> Cmd msg


port returnAdd : (Int -> msg) -> Sub msg


port callServer : { password: String, message: String } -> Cmd msg


port returnServer : (String -> msg) -> Sub msg


port callClient : { password: String, message: String } -> Cmd msg


port returnClient : (String -> msg) -> Sub msg


type alias Model =
    { a : String
    , b : String
    , result : Maybe Int
    , password : String
    , message : String
    , receivedFromClient : Maybe String
    , receivedFromServer : Maybe String
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model "" ""  Nothing "" "" Nothing Nothing, Cmd.none )


view : Model -> Browser.Document Msg
view model =
    { title = "D4FT4"
    , body =
        [ div []
            [ input [ placeholder "a", value model.a, onInput AChanged ] []
            , input [ placeholder "b", value model.b, onInput BChanged ] []
            , button [ onClick Calculate ] [ text "calculate" ]
            , model.result |> Maybe.map (String.fromInt >> text) |> Maybe.withDefault (text "no answer")
            ]
        , div []
            [ input [ type_ "password", placeholder "password", value model.password, onInput PasswordChanged ] []
            , input [ placeholder "message", value model.message, onInput MessageChanged ] []
            , button [ onClick CallServer ] [ text "set up server and send to client" ]
            , button [ onClick CallClient ] [ text "set up client and send to server" ]
            ]
        , div []
            (text "Received from client: "
                :: (model.receivedFromClient |> Maybe.map (text >> List.singleton) |> Maybe.withDefault [])
            )
        , div []
            (text "Received from server: "
                :: (model.receivedFromServer |> Maybe.map (text >> List.singleton) |> Maybe.withDefault [])
            )
        ]
    }


type Msg
    = AChanged String
    | BChanged String
    | Calculate
    | ReturnAdd Int
    | PasswordChanged String
    | MessageChanged String
    | CallServer
    | ReturnServer String
    | CallClient
    | ReturnClient String


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
        
        PasswordChanged password ->
            ( { model | password = password }, Cmd.none )

        MessageChanged message ->
            ( { model | message = message }, Cmd.none )

        CallServer ->
            ( model, callServer { password = model.password, message = model.message } )

        ReturnServer message ->
            ( { model | receivedFromClient = Just message }, Cmd.none )

        CallClient ->
            ( model, callClient { password = model.password, message = model.message } )

        ReturnClient message ->
            ( { model | receivedFromServer = Just message }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ returnAdd ReturnAdd
        , returnServer ReturnServer
        , returnClient ReturnClient
        ]


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
