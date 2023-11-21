module FileTransfer exposing (main)

import Browser
import Components exposing (ErrorMessage)
import Home
import Json.Decode
import Messaging
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
    , isErrorListOpen : Bool
    , errors : Components.ErrorMessageQueue
    }


init : String -> ( Model, Cmd Msg )
init platform =
    ( { page = Home
      , home = Home.init
      , send = Send.init
      , receive = Receive.init platform
      , isErrorListOpen = False
      , errors = Components.initErrorMessageQueue
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
            [ let
                viewToolbar maybeBackMsg convertMsg =
                    Components.viewToolbar
                        { maybeBackMsg = maybeBackMsg
                        , toggleErrorMsg = ErrorListToggled
                        , convertMsg = convertMsg
                        , isErrorListOpen = model.isErrorListOpen
                        , errors = model.errors
                        }
              in
              case model.page of
                Home ->
                    Home.view (PageChanged Send) (PageChanged Receive) HomeMsg (viewToolbar Nothing HomeMsg) model.home

                Send ->
                    Send.view SendMsg (viewToolbar (Just Back) SendMsg) model.send

                Receive ->
                    Receive.view ReceiveMsg (viewToolbar (Just Back) ReceiveMsg) model.receive
            ]
        ]
    }


type Msg
    = PageChanged Page
    | Back
    | ErrorListToggled
    | NewErrorMessage ErrorMessage
    | HomeMsg Home.Msg
    | SendMsg Send.Msg
    | ReceiveMsg Receive.Msg
    | ReceiveResponse (Result Json.Decode.Error (Messaging.Message Messaging.Response))


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        PageChanged page ->
            ( { model | page = page }, Cmd.none )

        Back ->
            update (PageChanged Home) model

        ErrorListToggled ->
            ( { model
                | isErrorListOpen = not model.isErrorListOpen
                , errors =
                    if model.isErrorListOpen then
                        Components.markErrorsRead model.errors

                    else
                        model.errors
              }
            , Cmd.none
            )

        NewErrorMessage error ->
            ( { model | errors = Components.addErrorMessage model.errors error }, Cmd.none )

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
                Ok ({ returnPath, message } as fullMessage) ->
                    let
                        modelWithError =
                            case message of
                                Messaging.Error error ->
                                    { model
                                        | errors =
                                            Components.addErrorMessage model.errors
                                                { source = returnPath
                                                , message = error
                                                }
                                    }

                                _ ->
                                    model
                    in
                    case returnPath of
                        "Send" :: pathTail ->
                            update (SendMsg <| Send.ReceiveResponse { fullMessage | returnPath = pathTail }) modelWithError

                        "Receive" :: pathTail ->
                            update (ReceiveMsg <| Receive.ReceiveResponse { fullMessage | returnPath = pathTail }) modelWithError

                        _ ->
                            ( model, Cmd.none )

                Err error ->
                    ( { model
                        | errors =
                            Components.addErrorMessage model.errors
                                { source = [ "Message Parsing" ]
                                , message = Json.Decode.errorToString error
                                }
                      }
                    , Cmd.none
                    )



-- ReceiveResponse maybeResponse ->
--     case maybeResponse of
--         Ok ({ returnPath, message } as response) ->
--             case ( returnPath, message ) of
--                 ( "Send" :: pathTail, Messaging.Error ) ->
--                     ( model, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ Sub.map SendMsg (Send.subscriptions model.send)
        , Sub.map ReceiveMsg (Receive.subscriptions model.receive)
        , Messaging.receiveBackendMessage ReceiveResponse
        ]


main : Program String Model Msg
main =
    Browser.document
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
