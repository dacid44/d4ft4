module Components exposing (ErrorMessage, ErrorMessageQueue, ToolbarArgs, addErrorMessage, initErrorMessageQueue, markErrorsRead, viewToolbar)

import Html exposing (..)
import Material.Icons as Filled
import Material.Icons.Types exposing (Coloring(..))
import Maybe.Extra as Maybe
import Theme
import W.Badge as Badge
import W.Button as Button
import W.Container as Container
import W.Divider as Divider
import W.Message as Message
import W.Popover as Popover


type alias ToolbarArgs parentMsg pageMsg =
    { maybeBackMsg : Maybe parentMsg
    , toggleErrorMsg : parentMsg
    , convertMsg : pageMsg -> parentMsg
    , isErrorListOpen : Bool
    , errors : ErrorMessageQueue
    }


viewToolbar : ToolbarArgs parentMsg pageMsg -> Html pageMsg -> Html parentMsg
viewToolbar args child =
    Container.view [ Container.horizontal, Container.gap_3 ]
        ((args.maybeBackMsg
            |> Maybe.map (\backMsg -> [ Button.view [ Button.icon ] { label = [ Filled.arrow_back 24 Inherit ], onClick = backMsg } ])
            |> Maybe.withDefault []
         )
            ++ [ Html.map args.convertMsg child
               , Container.view
                    [ Container.horizontal
                    , Container.styleAttrs [ ( "margin-left", "auto" ) ]
                    ]
                    [ Popover.viewControlled [ Popover.left, Popover.offset 8 ]
                        { isOpen = args.isErrorListOpen
                        , content = [ viewErrorList args.errors ]
                        , trigger =
                            [ Button.view
                                [ Button.icon
                                , Button.theme
                                    { foreground = Theme.neutralForeground
                                    , background = Theme.neutralBackground
                                    , aux = Theme.dangerForeground
                                    }
                                ]
                                { label =
                                    [ Badge.view [ Badge.small ]
                                        { content =
                                            args.errors.unread
                                                |> List.head
                                                |> Maybe.filter (\_ -> not args.isErrorListOpen)
                                                |> Maybe.map
                                                    (\_ ->
                                                        args.errors.unread
                                                            |> List.length
                                                            |> String.fromInt
                                                            |> text
                                                            |> List.singleton
                                                    )
                                        , children =
                                            if args.isErrorListOpen then
                                                [ Filled.close 24 Inherit ]

                                            else
                                                [ Filled.report 24 Inherit ]
                                        }
                                    ]
                                , onClick = args.toggleErrorMsg
                                }
                            ]
                        }
                    ]
               ]
        )


type alias ErrorMessage =
    { source : List String, message : String }


type alias ErrorMessageQueue =
    { read : List ErrorMessage, unread : List ErrorMessage }


initErrorMessageQueue : ErrorMessageQueue
initErrorMessageQueue =
    { read = [], unread = [] }


addErrorMessage : ErrorMessageQueue -> ErrorMessage -> ErrorMessageQueue
addErrorMessage errors newError =
    { errors | unread = errors.unread ++ [ newError ] }


markErrorsRead : ErrorMessageQueue -> ErrorMessageQueue
markErrorsRead errors =
    { read = errors.read ++ errors.unread, unread = [] }


viewErrorList : ErrorMessageQueue -> Html msg
viewErrorList errors =
    Container.view
        [ Container.vertical
        , Container.card
        , Container.background Theme.neutralBackground
        , Container.pad_3
        , Container.gap_2
        , Container.styleAttrs
            [ ( "min-width", "40vw" )
            , ( "width", "30rem" )
            , ( "max-width", "calc(100vw - 32px - 3rem)" )
            , ( "max-height", "calc(100vh - 2rem)" )
            , ( "overflow-y", "auto" )
            ]
        ]
        (case ( List.isEmpty errors.read, List.isEmpty errors.unread ) of
            ( True, True ) ->
                [ Message.view [ Message.success ] [ text "No errors!" ] ]

            ( True, False ) ->
                List.map viewErrorMessage errors.unread

            ( False, True ) ->
                List.map viewErrorMessage errors.read

            ( False, False ) ->
                List.map viewErrorMessage errors.read
                    ++ [ Divider.view [ Divider.color Theme.primaryForeground ] [ text "New" ] ]
                    ++ List.map viewErrorMessage errors.unread
        )


viewErrorMessage : ErrorMessage -> Html msg
viewErrorMessage error =
    Message.view
        [ Message.danger, Message.footer [ text error.message ] ]
        [ text <| String.join " / " error.source ]
