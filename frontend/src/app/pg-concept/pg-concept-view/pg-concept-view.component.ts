import { CommonModule } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';
import { TranslateService } from '@ngx-translate/core';

import { ConceptViewComponent } from 'src/app/lib-concept/concept-view/concept-view.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ConfirmationData } from 'src/app/lib-dialog/confirmation/confirmation.component';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';
import { StreamDto, StreamState, StreamStateUtil } from 'src/app/lib-stream/stream-api.interface';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

@Component({
    selector: 'app-pg-concept-view',
    standalone: true,
    imports: [CommonModule, ConceptViewComponent],
    templateUrl: './pg-concept-view.component.html',
    styleUrl: './pg-concept-view.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgConceptViewComponent implements OnInit {

    public isLoadStream = false;
    public isShowTimer: boolean = false;
    // An indication that the stream is in active status. ([preparing, started, paused]) 
    public isStreamActive: boolean = false;
    // An indication that this is the owner of the stream.
    public isStreamOwner = false;
    // The interval for displaying the timer before starting (in minutes).
    public showTimerBeforeStart: number | null | undefined;

    public profileDto: ProfileDto | null = null;
    public streamDto: StreamDto | null = null;

    constructor(
        private changeDetector: ChangeDetectorRef,
        private route: ActivatedRoute,
        private translateService: TranslateService,
        private alertService: AlertService,
        private dialogService: DialogService,
        private streamService: StreamService,
    ) {
        // this.showTimerBeforeStart = 120; // minutes
        this.profileDto = this.route.snapshot.data['profileDto'];
        this.streamDto = this.route.snapshot.data['streamDto'];

        if (!!this.streamDto) { // #
            // this.streamDto.state = StreamState.waiting; // # for demo
            // this.streamDto.state = StreamState.preparing; // # for demo
            this.streamDto.state = StreamState.started; // # for demo
            // this.streamDto.state = StreamState.paused; // # for demo
            // this.streamDto.state = StreamState.stopped; // # for demo
            // #  this.streamDto.starttime = "2024-10-29T16:34:00.000Z";
        }
    }

    ngOnInit(): void {
        this.updateParams(this.streamDto, this.profileDto);
    }

    // ** Public API **

    public doChangeState(isStreamOwner: boolean, streamId: number | undefined, newState: StreamState | null): void {
        if (!!isStreamOwner && !!streamId && !!newState) {
            this.toggleStreamState(isStreamOwner, streamId, newState);
        }

    }

    // ** Private API **

    private updateParams(streamDto: StreamDto | null, profileDto: ProfileDto | null): void {
        let isShowTimer = false;
        let isStreamOwner = false;
        if (!!streamDto) {
            isShowTimer = !!streamDto && StreamStateUtil.isActive(streamDto.state);
            const currentUserId: number = profileDto?.id || -1;
            isStreamOwner = (streamDto.userId === currentUserId);
        }
        this.isShowTimer = isShowTimer;
        this.isStreamOwner = isStreamOwner;
    }

    // Stream for Owner

    private toggleStreamState(isStreamOwner: boolean, streamId: number | null, streamState: StreamState | null): void {
        if (!isStreamOwner || !streamId || !streamState) {
            return;
        }
        this.isLoadStream = true;
        this.streamService.toggleStreamState(streamId, streamState)
            .then((response: StreamDto | HttpErrorResponse) => {
                this.streamDto = (response as StreamDto);
                this.updateParams(this.streamDto, this.profileDto);
            })
            .catch((error: HttpErrorResponse) => {
                const appError = (typeof (error?.error || '') == 'object' ? error.error : {});
                const title = 'pg-concept-view.error_update_stream';

                if (error.status == 409 && appError['code'] == 'Conflict' && appError['message'] == 'exist_is_active_stream') {
                    const errParams = appError['params'] || {};
                    const link = this.streamService.getLinkForVisitors(errParams['activeStream']['id'] || -1, false);
                    const name = errParams['activeStream']['title'] || '';
                    const confirmData: ConfirmationData = {
                        messageHtml: this.translateService.instant('pg-concept-view.exist_is_active_stream', { link, name }),
                    };
                    this.dialogService.openConfirmation(
                        '', title, { btnNameCancel: null, btnNameAccept: 'buttons.ok' }, { data: confirmData });
                } else {
                    this.alertService.showError(HttpErrorUtil.getMsgs(error)[0], title);
                }
            })
            .finally(() => {
                this.isLoadStream = false;
                this.changeDetector.markForCheck();
            });

    }

}
