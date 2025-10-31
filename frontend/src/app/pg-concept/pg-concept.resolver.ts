import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { P_CONCEPT_ID, E_CONCEPT_VIEW } from '../common/routes';
import { StreamService } from '../lib-stream/stream.service';
import { StreamDto } from '../lib-stream/stream-api.interface';
import { BlockedUserDto } from '../lib-chat/chat-message-api.interface';
import { ChatMessageService } from '../lib-chat/chat-message.service';
import { ProfileService } from '../lib-profile/profile.service';
import { ProfileMiniDto, ProfileMiniDtoUtil } from '../lib-profile/profile-api.interface';

function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl('/technical/not-found').then(() => Promise.resolve(undefined));
}

export interface ConceptResponse {
    streamDto: StreamDto;
    profileMiniDto: ProfileMiniDto;
    blockedNames: string[];
}

export const pgConceptResolver: ResolveFn<ConceptResponse | HttpErrorResponse | undefined> =
    (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
        const router = inject(Router);
        const chatMessageService: ChatMessageService = inject(ChatMessageService);
        const profileService: ProfileService = inject(ProfileService);
        const streamService: StreamService = inject(StreamService);

        const profileDto = profileService.profileDto;
        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_CONCEPT_ID);
        const streamId = parseInt(streamIdStr || '-1', 10);

        if (E_CONCEPT_VIEW === url.path && streamId > -1) {
            return streamService.getStream(streamId)
                .then(async (response: StreamDto | HttpErrorResponse | undefined) => {
                    const streamDto: StreamDto = (response as StreamDto);
                    let profileMiniDto: ProfileMiniDto = ProfileMiniDtoUtil.create({});
                    const blockedNames: string[] = [];

                    const buffPromise: Promise<unknown>[] = [];
                    // Get a mini profile of the stream owner.
                    buffPromise.push(profileService.profileMini(streamDto.userId));

                    if (!!profileDto?.id && profileDto.id == streamDto.userId) {
                        // Get a list of blocked users.
                        buffPromise.push(chatMessageService.getBlockedNames());
                    }
                    try {
                        const responses = await Promise.all(buffPromise);
                        profileMiniDto = responses[0] as ProfileMiniDto;
                        if (!!responses[1]) {
                            blockedNames.push(...(responses[1] as string[]));
                        }
                    } catch (error) {
                        goToPageNotFound(router);
                    }

                    return { streamDto, profileMiniDto, blockedNames };
                })
                .catch((err) =>
                    goToPageNotFound(router));
        } else {
            return goToPageNotFound(router);
        }
    };
