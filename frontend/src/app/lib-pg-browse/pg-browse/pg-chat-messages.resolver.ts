import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ResolveFn, ActivatedRouteSnapshot, RouterStateSnapshot } from "@angular/router";
import { P_BROWSE_ID, E_BROWSE_VIEW } from "../../common/routes";
import { SessionSrv } from "../../common/session-srv";
import { ChatMessageDto } from "../../lib-chat/chat-message";
import { ChatMessageSrv } from "../../lib-chat/chat-message-srv";

export const pgChatMessagesResolver: ResolveFn<ChatMessageDto[] | HttpErrorResponse | undefined>
    = (route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const chatMessageSrv: ChatMessageSrv = inject(ChatMessageSrv);
        const sessionSrv = inject(SessionSrv);

        if (!sessionSrv.getUser()) {
            return [];
        }

        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_BROWSE_ID);
        const streamId = parseInt(streamIdStr || "-1", 10);

        if (E_BROWSE_VIEW === url.path && streamId > -1) {
            return chatMessageSrv.getChatMessages(streamId, true)
                .catch(() => []);
        } else {
            return [];
        }
    };
