module FileTransfer exposing (main)

import Browser
import Common
import Home
import Json.Decode
import Receive
import Send
import Theme
import W.Container as Container
import W.Styles


type Page
    = Home
    | Send
    | Receive


type alias Model =
    { page : Page
    , home : Home.Model
    , send : Send.Model
    , receive : Receive.Model
    }


init : String -> ( Model, Cmd Msg )
init platform =
    ( { page = Home
      , home = Home.init
      , send = Send.init
      , receive = Receive.init platform
      }
    , Cmd.none
    )


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
        , Container.view
            [ Container.vertical
            , Container.background Theme.baseBackground
            , Container.styleAttrs [ ( "height", "100%" ) ]
            ]
            [ case model.page of
                Home ->
                    Home.view (PageChanged Send) (PageChanged Receive) HomeMsg model.home

                Send ->
                    Send.view (PageChanged Home) SendMsg model.send

                Receive ->
                    Receive.view (PageChanged Home) ReceiveMsg model.receive
            ]
        ]
    }


type Msg
    = PageChanged Page
    | HomeMsg Home.Msg
    | SendMsg Send.Msg
    | ReceiveMsg Receive.Msg
    | ReceiveResponse (Result Json.Decode.Error (Common.Message Common.Response))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        PageChanged page ->
            ( { model | page = page }, Cmd.none )

        HomeMsg subMsg ->
            let
                ( subModel, subCmd ) =
                    Home.update subMsg model.home
            in
            ( { model | home = subModel }, Cmd.map HomeMsg subCmd )

        SendMsg subMsg ->
            let
                ( subModel, subCmd ) =
                    Send.update subMsg model.send
            in
            ( { model | send = subModel }, Cmd.map SendMsg subCmd )

        ReceiveMsg subMsg ->
            let
                ( subModel, subCmd ) =
                    Receive.update subMsg model.receive
            in
            ( { model | receive = subModel }, Cmd.map ReceiveMsg subCmd )

        ReceiveResponse maybeMessage ->
            case maybeMessage of
                Ok ({ returnPath } as fullMessage) ->
                    case returnPath of
                        "Send" :: pathTail ->
                            update (SendMsg <| Send.ReceiveResponse { fullMessage | returnPath = pathTail }) model

                        "Receive" :: pathTail ->
                            update (ReceiveMsg <| Receive.ReceiveResponse { fullMessage | returnPath = pathTail }) model

                        _ ->
                            ( model, Cmd.none )

                Err _ ->
                    ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Sub.map SendMsg (Send.subscriptions model.send)
        , Sub.map ReceiveMsg (Receive.subscriptions model.receive)
        , Common.receiveBackendMessage ReceiveResponse
        ]


main : Program String Model Msg
main =
    Browser.document
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
