import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { P_STREAM_ID, E_STREAM_EDIT, E_STREAM_CREATE } from '../common/routes';
import { StreamService } from '../lib-stream/stream.service';
import { StreamDto, StreamDtoUtil } from '../lib-stream/stream-api.interface';

function goToPageNotFound(router: Router): Promise<undefined> {
    return router.navigateByUrl('/technical/not-found').then(() => Promise.resolve(undefined));
}

export const pgStreamResolver: ResolveFn<StreamDto | HttpErrorResponse | undefined> =
    (route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
        const router = inject(Router);
        const streamService: StreamService = inject(StreamService);

        const url = route.url[0];
        const streamIdStr = route.paramMap.get(P_STREAM_ID);
        const streamId = parseInt(streamIdStr || '-1', 10);
        if (E_STREAM_CREATE === url.path) {
            const streamIdForDuplicationStr = route.queryParamMap.get('id');
            const streamIdForDuplication = parseInt(streamIdForDuplicationStr || '-1', 10);
            // If the "ID" parameter is specified, then this stream for duplication.
            if (streamIdForDuplication > 0) {
                return streamService.getStream(streamIdForDuplication)
                    .then((response: StreamDto | HttpErrorResponse | undefined) => {
                        (response as StreamDto).id = -1;
                        return response;
                    })
                    .catch((err) =>
                        goToPageNotFound(router));
            } else {
                return Promise.resolve(StreamDtoUtil.create());
            }
        } else if (E_STREAM_EDIT === url.path && !!streamId) {
            return streamService.getStream(streamId)
                .catch((err) =>
                    goToPageNotFound(router));
        } else {
            return goToPageNotFound(router);
        }
    };
