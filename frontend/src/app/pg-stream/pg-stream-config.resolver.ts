import { ActivatedRouteSnapshot, ResolveFn, RouterStateSnapshot } from '@angular/router';
import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';

import { StreamConfigDto } from '../lib-stream/stream-config.interface';
import { StreamConfigService } from '../lib-stream/stream-config.service';

export const pgStreamConfigResolver: ResolveFn<StreamConfigDto | HttpErrorResponse | undefined> =
    (_route: ActivatedRouteSnapshot, _state: RouterStateSnapshot) => {
        const streamConfigService: StreamConfigService = inject(StreamConfigService);
        return streamConfigService.getConfig();
    };
