port module Main exposing (main)

import Browser
import Html exposing (..)
import Html.Attributes exposing (placeholder, value)
import Html.Events exposing (onClick, onInput)
import Html.Attributes exposing (type_)
import Array exposing (Array)
import Html.Attributes exposing (selected)
import Maybe exposing (withDefault)


port callSetup : { connId: Int, isServer: Bool, mode: String, password: String } -> Cmd msg
port callSendText : { connId: Int, text: String } -> Cmd msg
port callReceiveText : { connId: Int } -> Cmd msg
port callSendFile : { connId: Int, path: String } -> Cmd msg
port callReceiveFile : { connId: Int, path: String } -> Cmd msg

port returnMessage : (( Int, Maybe String ) -> msg) -> Sub msg

port callSelectFile : { connId: Int, save: Bool } -> Cmd msg
port returnSelectFile : (( Int, Maybe String ) -> msg) -> Sub msg


type alias Model =
    Array ConnectionModel

type alias ConnectionModel =
    { password : String
    , mode : TransferMode
    , text : String
    , path : String
    , receivedMessage : Maybe String
    }

type TransferMode
    = SendText
    | ReceiveText
    | SendFile
    | ReceiveFile

encodeTransferMode : TransferMode -> String
encodeTransferMode mode =
    case mode of
        SendText -> "send-text"
        ReceiveText -> "receive-text"
        SendFile -> "send-file"
        ReceiveFile -> "receive-file"

decodeTransferMode : String -> TransferMode
decodeTransferMode mode =
    case mode of
        "send-text" -> SendText
        "receive-text" -> ReceiveText
        "send-file" -> SendFile
        "receive-file" -> ReceiveFile
        _ -> SendText


init : () -> ( Model, Cmd Msg )
init _ =
    ( Array.repeat 2 (ConnectionModel "" SendText "" "" Nothing), Cmd.none )


view : Model -> Browser.Document Msg
view model =
    { title = "D4FT4"
    , body = model |> Array.indexedMap connectionPanel |> Array.toList
    }

connectionPanel : Int -> ConnectionModel -> Html Msg
connectionPanel connId connectionModel =
    div []
        [ div []
            [ input [ type_ "password", placeholder "password", value connectionModel.password, onInput (PasswordChanged connId) ] []
            , select [ value (encodeTransferMode connectionModel.mode), onInput (TransferModeChanged connId << decodeTransferMode) ]
                [ option [ value "send-text", selected True ] [ text "Send text" ]
                , option [ value "receive-text" ] [ text "Receive text" ]
                , option [ value "send-file" ] [ text "Send file" ]
                , option [ value "receive-file" ] [ text "Receive file" ]
                ]
            , button [ onClick (CallSetupServer connId) ] [ text "Set up server" ]
            , button [ onClick (CallSetupClient connId) ] [ text "Set up client" ]
            ]
        , div []
            [ input [ placeholder "text", value connectionModel.text, onInput (TextChanged connId) ] []
            , button [ onClick (CallSendText connId) ] [ text "Send text" ]
            , button [ onClick (CallReceiveText connId) ] [ text "Receive text" ]
            ]
        , div []
            [ input [ placeholder "path", value connectionModel.path, onInput (PathChanged connId) ] []
            , button [ onClick (SelectFile connId False ) ] [ text "Select file" ]
            , button [ onClick (SelectFile connId True) ] [ text "Select file (save)" ]
            , button [ onClick (CallSendFile connId) ] [ text "Send file" ]
            , button [ onClick (CallReceiveFile connId) ] [ text "Receive file" ]
            ]
        , div [] [ text <| "Message: " ++ (connectionModel.receivedMessage |> withDefault "") ]
        ]


type Msg
    = PasswordChanged Int String
    | TextChanged Int String
    | TransferModeChanged Int TransferMode
    | SelectFile Int Bool
    | PathChanged Int String
    | CallSetupServer Int
    | CallSetupClient Int
    | CallSendText Int
    | CallReceiveText Int
    | CallSendFile Int
    | CallReceiveFile Int
    | ReturnMessage Int (Maybe String)


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        PasswordChanged connId password ->
            ( updateModelArray connId model (\connectionModel -> { connectionModel | password = password }), Cmd.none )

        TextChanged connId text ->
            ( updateModelArray connId model (\connectionModel -> { connectionModel | text = text }), Cmd.none )

        TransferModeChanged connId mode ->
            ( updateModelArray connId model (\connectionModel -> { connectionModel | mode = mode }), Cmd.none )
        
        SelectFile connId save ->
            ( model, callSelectFile { connId = connId, save = save })

        PathChanged connId path ->
            ( updateModelArray connId model (\connectionModel -> { connectionModel | path = path }), Cmd.none )

        CallSetupServer connId ->
            ( model
            , callSetup
                { connId = connId
                , isServer = True
                , mode = model |> Array.get connId |> Maybe.map (.mode >> encodeTransferMode) |> Maybe.withDefault "send-text"
                , password = model |> Array.get connId |> Maybe.map .password |> Maybe.withDefault ""
                }
            )
        
        CallSetupClient connId ->
            ( model
            , callSetup
                { connId = connId
                , isServer = False
                , mode = model |> Array.get connId |> Maybe.map (.mode >> encodeTransferMode) |> Maybe.withDefault "send-text"
                , password = model |> Array.get connId |> Maybe.map .password |> Maybe.withDefault ""
                }
            )

        CallSendText connId ->
            ( model
            , callSendText
                { connId = connId
                , text = model |> Array.get connId |> Maybe.map .text |> Maybe.withDefault ""
                }
            )

        CallReceiveText connId ->
            ( model, callReceiveText { connId = connId } )

        CallSendFile connId ->
            ( model
            , callSendFile
                { connId = connId
                , path = model |> Array.get connId |> Maybe.map .path |> Maybe.withDefault ""
                }
            )

        CallReceiveFile connId ->
            ( model
            , callReceiveFile
                { connId = connId
                , path = model |> Array.get connId |> Maybe.map .path |> Maybe.withDefault ""
                }
            )

        ReturnMessage connId message ->
            ( updateModelArray connId model (\connectionModel -> { connectionModel | receivedMessage = message }), Cmd.none )

updateModelArray : Int -> Model -> (ConnectionModel -> ConnectionModel) -> Model
updateModelArray connId model mapFn =
    model |> Array.indexedMap (\i connectionModel -> if i == connId then mapFn connectionModel else connectionModel)


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch
        [ returnMessage (\( connId, message ) -> ReturnMessage connId message)
        , returnSelectFile (\( connId, path ) ->
            case path of
                Just newPath ->
                    PathChanged connId newPath
                Nothing ->
                    PathChanged -1 ""
            )
        ]


main : Program () Model Msg
main =
    Browser.document
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }
