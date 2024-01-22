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
  let router = inject(Router);
  let streamService: StreamService = inject(StreamService);

  const url = route.url[0];
  const streamId = route.paramMap.get(P_STREAM_ID);
  if (E_STREAM_CREATE === url.path) {
    const dublicateStreamId = route.queryParamMap.get('id');
    if (!!dublicateStreamId) {
      return streamService.getStream(dublicateStreamId)
        .then((response: StreamDto | HttpErrorResponse | undefined) => {
          (response as StreamDto).id = '';
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
