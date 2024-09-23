import { ActivatedRouteSnapshot, ResolveFn, Router, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { P_CONCEPT_ID, E_CONCEPT_VIEW } from '../common/routes';
import { StreamService } from '../lib-stream/stream.service';
import { StreamDto } from '../lib-stream/stream-api.interface';

function goToPageNotFound(router: Router): Promise<undefined> {
  return router.navigateByUrl('/technical/not-found').then(() => Promise.resolve(undefined));
}

export const pgConceptResolver: ResolveFn<StreamDto | HttpErrorResponse | undefined> = 
(route: ActivatedRouteSnapshot, state: RouterStateSnapshot) => {
  const router = inject(Router);
  const streamService: StreamService = inject(StreamService);

  const url = route.url[0];
  const streamIdStr = route.paramMap.get(P_CONCEPT_ID);
  const streamId = parseInt(streamIdStr || '-1', 10);

  if (E_CONCEPT_VIEW === url.path && !!streamId) {
    return streamService.getStream(streamId)
      .catch((err) =>
        goToPageNotFound(router));
  } else {
    return goToPageNotFound(router);
  }
};
