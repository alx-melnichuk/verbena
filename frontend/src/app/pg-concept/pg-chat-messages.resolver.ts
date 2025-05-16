import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { P_CONCEPT_ID, E_CONCEPT_VIEW } from '../common/routes';
import { ChatMessageDto } from '../lib-chat/chat-message-api.interface';
import { ChatMessageService } from '../lib-chat/chat-message.service';
import { ProfileService } from '../lib-profile/profile.service';

function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl('/technical/not-found').then(() => Promise.resolve(undefined));
}

export const pgChatMessagesResolver: ResolveFn<ChatMessageDto[] | HttpErrorResponse | undefined> =
    (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
        const router = inject(Router);
        const chatMessageService: ChatMessageService = inject(ChatMessageService);
        const profileService: ProfileService = inject(ProfileService);

        if (!profileService.profileDto) {
            return [];
        }

        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_CONCEPT_ID);
        const streamId = parseInt(streamIdStr || '-1', 10);

        if (E_CONCEPT_VIEW === url.path && streamId > -1) {
            return chatMessageService.getChatMessages(streamId, true)
                .catch((err) =>
                    goToPageNotFound(router));
        } else {
            return goToPageNotFound(router);
        }
    };
