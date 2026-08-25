import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from "@angular/router";
import { HttpErrorResponse } from "@angular/common/http";
import { inject } from "@angular/core";

import { ChatMessageSrv } from "../../lib-chat/chat-message-srv";
import { BlockedUserDto } from "../../lib-chat/chat-message";
import { SORT_COL_INIT, SORT_DESC_INIT } from "./pg-banned";

function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl("/technical/not-found").then(() => Promise.resolve(undefined));
}

export const pgBannedUsersResolver: ResolveFn<BlockedUserDto[] | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const router = inject(Router);
        const chatMessageService: ChatMessageSrv = inject(ChatMessageSrv);

        let sortColumn: string = SORT_COL_INIT;
        let sortDesc: boolean = SORT_DESC_INIT;
        return chatMessageService.getBlockedUsers(sortColumn, sortDesc)
            .catch((err) => {
                console.error(err);
                return goToPageNotFound(router);
            });
    };

