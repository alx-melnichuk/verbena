import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";
import { ResolveFn, ActivatedRouteSnapshot, RouterStateSnapshot } from "@angular/router";
import { P_BROWSE_ID, E_BROWSE_VIEW } from "../../common/routes";
import { SessionSrv } from "../../common/session-srv";
import { ChatMessageSrv } from "../../lib-chat/chat-message-srv";
import { StreamDto } from "../../lib-stream/stream-dto";
import { StreamSrv } from "../../lib-stream/stream-srv";
import { UserShortDto } from "../../lib-user/user-dto";
import { UserSrv } from "../../lib-user/user-srv";

export interface BrowseStream {
    streamDto: StreamDto;
    userShortDto: UserShortDto;
    blockedNames: string[];
}

export const pgBrowseStreamResolver: ResolveFn<BrowseStream | HttpErrorResponse | undefined>
    = (route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const chatMessageSrv: ChatMessageSrv = inject(ChatMessageSrv);
        const sessionSrv = inject(SessionSrv);
        const streamSrv: StreamSrv = inject(StreamSrv);
        const userSrv: UserSrv = inject(UserSrv);

        const currUser = sessionSrv.getUser();
        if (!currUser) {
            return undefined;
        }

        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_BROWSE_ID);
        const streamId = parseInt(streamIdStr || "-1", 10);

        if (E_BROWSE_VIEW === url.path && streamId > -1) {
            return streamSrv.getStream(streamId)
                .then((response: StreamDto | HttpErrorResponse | undefined) => {
                    const streamDto: StreamDto = (response as StreamDto);
                    const blockedNames: string[] = [];
                    const buffPromise: Promise<unknown>[] = [];
                    // Get a mini profile of the stream owner.
                    buffPromise.push(userSrv.getUserShort(streamDto.userId));

                    if (currUser.id == streamDto.userId) {
                        buffPromise.push(chatMessageSrv.getBlockedUsersNames()); // Get a list of blocked users.
                    }

                    return Promise.all(buffPromise)
                        .then((responses) => {
                            const userShortDto: UserShortDto = (responses[0] as UserShortDto);
                            if (!!responses[1]) {
                                blockedNames.push(...(responses[1] as string[]));
                            }
                            return { streamDto, userShortDto, blockedNames };
                        })
                        .catch(() => undefined);
                })
                .catch(() => undefined);
        } else {
            return undefined;
        }
    };
