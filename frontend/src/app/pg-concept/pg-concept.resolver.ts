import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { P_CONCEPT_ID, E_CONCEPT_VIEW } from '../common/routes';
import { StreamService } from '../lib-stream/stream.service';
import { StreamDto } from '../lib-stream/stream-api.interface';
import { BlockedUserDto } from '../lib-chat/chat-message-api.interface';
import { ChatMessageService } from '../lib-chat/chat-message.service';
import { ProfileService } from '../lib-profile/profile.service';

function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl('/technical/not-found').then(() => Promise.resolve(undefined));
}

export interface ConceptResponse {
    streamDto: StreamDto;
    blockedUsersDto: BlockedUserDto[];
}

export const pgConceptResolver: ResolveFn<ConceptResponse | HttpErrorResponse | undefined> =
    (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
        const router = inject(Router);
        const chatMessageService: ChatMessageService = inject(ChatMessageService);
        const profileService: ProfileService = inject(ProfileService);
        const streamService: StreamService = inject(StreamService);

        const profileDto = profileService.profileDto;
        if (!profileDto) {
            return undefined;
        }

        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_CONCEPT_ID);
        const streamId = parseInt(streamIdStr || '-1', 10);

        if (E_CONCEPT_VIEW === url.path && streamId > -1) {
            return streamService.getStream(streamId)
                .then((response: StreamDto | HttpErrorResponse | undefined) => {
                    const streamDto: StreamDto = (response as StreamDto);
                    if (streamDto.userId == profileDto.id) {
                        return chatMessageService.getBlockedUsers()
                            .then((response: BlockedUserDto[] | HttpErrorResponse | undefined) => {
                                const blockedUsersDto: BlockedUserDto[] = (response as BlockedUserDto[]);
                                return { streamDto, blockedUsersDto };
                            })
                            .catch((err) =>
                                goToPageNotFound(router));
                    }
                    return { streamDto, blockedUsersDto: [] };
                })
                .catch((err) =>
                    goToPageNotFound(router));
        } else {
            return goToPageNotFound(router);
        }
    };
