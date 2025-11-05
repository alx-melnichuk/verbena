import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { ChatMessageService } from '../lib-chat/chat-message.service';
import { BlockedUserDto } from '../lib-chat/chat-message-api.interface';

function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl('/technical/not-found').then(() => Promise.resolve(undefined));
}

export const pgBannedResolver: ResolveFn<BlockedUserDto[] | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const router = inject(Router);
        const chatMessageService: ChatMessageService = inject(ChatMessageService);

        let sortColumn: string | undefined;
        let sortDesc: boolean | undefined;
        return chatMessageService.getBlockedUsers(sortColumn, sortDesc)
            .catch((err) => {
                console.error(err);
                return goToPageNotFound(router);
            });
    };
