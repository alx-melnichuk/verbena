import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { ROUTE_STREAM_EDIT, ROUTE_STREAM_LIST } from 'src/app/common/routes';
import { StreamDto, UpdateStreamFileDto } from 'src/app/lib-stream/stream-api.interface';
import { StreamConfigDto } from 'src/app/lib-stream/stream-config.interface';

import { StreamEditComponent } from 'src/app/lib-stream/stream-edit/stream-edit.component';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

@Component({
    selector: 'app-pg-stream-edit',
    standalone: true,
    imports: [CommonModule, StreamEditComponent],
    templateUrl: './pg-stream-edit.component.html',
    styleUrl: './pg-stream-edit.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamEditComponent {
    public errMsgs: string[] = [];
    public isLoadStream = false;
    public streamDto: StreamDto | null = null;
    public streamConfigDto: StreamConfigDto | null = null;

    private goBackToRoute: string = ROUTE_STREAM_LIST;

    constructor(
        private route: ActivatedRoute,
        private router: Router,
        private streamService: StreamService,

    ) {
        this.streamDto = this.route.snapshot.data['streamDto'];
        this.streamConfigDto = this.route.snapshot.data['streamConfigDto'];

        const previousNav = this.router.getCurrentNavigation()?.previousNavigation?.finalUrl?.toString() || '';
        if (!!previousNav && !previousNav.startsWith(ROUTE_STREAM_EDIT)) {
            this.goBackToRoute = previousNav;
        }
    }

    // ** Public API **

    public doUpdateStream(updateStreamFileDto: UpdateStreamFileDto | null): void {
        if (!updateStreamFileDto) {
            return;
        }
        // Checking if there are any non-empty fields.
        const key = 0; const value = 1; // entry=[key, value] -> entry[0]-key, entry[1]-value
        const is_all_empty = Object.entries(updateStreamFileDto)
            .findIndex((entry) => entry[key] != 'id' && entry[value] !== undefined) == -1;
        if (is_all_empty) { // If no fields are specified for update, exit.
            this.goBack();
            return;
        }

        const buffPromise: Promise<unknown>[] = [];
        this.isLoadStream = true;

        if (!!updateStreamFileDto.id) {
            buffPromise.push(this.streamService.modifyStream(updateStreamFileDto.id, updateStreamFileDto));
        } else {
            buffPromise.push(this.streamService.createStream(updateStreamFileDto));
        }
        Promise.all(buffPromise)
            .then(() => {
                Promise.resolve()
                    .then(() => {
                        this.goBack();
                    });
            })
            .catch((error: HttpErrorResponse) => {
                this.errMsgs = HttpErrorUtil.getMsgs(error);
                throw error;
            })
            .finally(() => {
                this.isLoadStream = false;
            });
    }

    // ** Private API **

    private goBack() {
        window.setTimeout(() => this.router.navigateByUrl(this.goBackToRoute), 0);
    }

}
