import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, OnInit, ViewEncapsulation } from '@angular/core';
import { ActivatedRoute } from '@angular/router';

import { AlertService } from 'src/app/lib-dialog/alert.service';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';
import { StreamCalendarService } from 'src/app/lib-stream/stream-calendar.service';
import { CN_DELETE_STREAM, StreamListComponent } from 'src/app/lib-stream/stream-list/stream-list.component';
import { StreamService } from 'src/app/lib-stream/stream.service';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

import { StreamHandler } from './stream-handler';

const CN_DEFAULT_LIMIT = 7;
const CN_INTERVAL_MINUTES = 5;

@Component({
    selector: 'app-pg-stream-list',
    standalone: true,
    imports: [CommonModule, StreamListComponent],
    templateUrl: './pg-stream-list.component.html',
    styleUrl: './pg-stream-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamListComponent implements OnInit {
    public errObj: KeyValue<string, string> | null = null;
    public isRefreshStreamEvent: boolean | null | undefined;
    public profileDto: ProfileDto | null;

    // "Future Streams"
    public futureStreamHdlr: StreamHandler;
    // "Past Streams"
    public pastStreamHdlr: StreamHandler;

    constructor(
        private changeDetector: ChangeDetectorRef,
        private route: ActivatedRoute,
        private alertService: AlertService,
        private streamService: StreamService,
        public streamCalendarService: StreamCalendarService,
    ) {
        this.profileDto = this.route.snapshot.data['profileDto'];
        this.futureStreamHdlr = new StreamHandler(this.streamService, true, CN_DEFAULT_LIMIT, CN_INTERVAL_MINUTES);
        this.pastStreamHdlr = new StreamHandler(this.streamService, false, CN_DEFAULT_LIMIT, CN_INTERVAL_MINUTES);
    }

    async ngOnInit(): Promise<void> {
        await this.loadFutureAndPastStreamsAndSchedule();
    }
    // ** Public API **

    // ** "Streams Calendar" panel-stream-calendar **

    public async doCalendarForPeriod(calendarStart: Date): Promise<void> {
        const buffPromise: Promise<unknown>[] = [];
        buffPromise.push( // Get a list of events (streams) for a specified date.
            this.streamCalendarService.getCalendarInfoForPeriod(calendarStart, false)
        );
        if (this.streamCalendarService.isShowEvents(calendarStart)) {
            buffPromise.push( // Get the next page of the list of short streams for the selected date.
                this.streamCalendarService.getListEventsForDate(this.streamCalendarService.eventsOfDaySelected, 1)
            );
        } else {
            this.streamCalendarService.clearStreamsEvent();
        }
        try {
            await Promise.all(buffPromise)
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** "Streams Event" panel-stream-event **

    public async doStreamEventsForDate(info: { selectedDate: Date | null, pageNum: number }): Promise<void> {
        try {
            // Get the next page of the list of short streams for the selected date.
            await this.streamCalendarService.getListEventsForDate(info.selectedDate, info.pageNum);
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** "Future Stream" and "Past Stream" panel-stream-info **

    public async doSearchNextFutureStream(): Promise<void> {
        try {
            // Get the next page of the "Future Stream".
            await this.futureStreamHdlr.searchNextStream();
            // Refresh view for "panel-stream-event".
            this.isRefreshStreamEvent = !this.isRefreshStreamEvent ? true : undefined;
        } catch (error) {
            this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_get_future_streams');
            throw error;
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    public async doSearchNextPastStream(): Promise<void> {
        try {
            // Get the next page of "Past Stream".
            await this.pastStreamHdlr.searchNextStream();
            // Refresh view for "panel-stream-event".
            this.isRefreshStreamEvent = !this.isRefreshStreamEvent ? true : undefined;
        } catch (error) {
            this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_get_past_streams');
            throw error;
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.streamService.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.streamService.redirectToStreamEditingPage(streamId);
    }

    public doActionView(streamId: number): void {
        this.streamService.redirectToStreamViewPage(streamId);
    }

    public async doActionDelete(streamId: number): Promise<void> {
        await this.deleteDataStream(streamId);
    }

    // ** Private API **

    private async loadFutureAndPastStreamsAndSchedule(): Promise<void> {
        // Clear array of "Future Stream"
        this.futureStreamHdlr.clearStream();
        // Clear array of "Past Stream"
        this.pastStreamHdlr.clearStream();

        const buffPromise: Promise<unknown>[] = [];
        const now = new Date();
        now.setHours(0, 0, 0, 0);

        // Get the next page of the "Future Stream".
        buffPromise.push(this.futureStreamHdlr.searchNextStream());
        // Get the next page of "Past Stream".
        buffPromise.push(this.pastStreamHdlr.searchNextStream());
        const selectedDate = this.streamCalendarService.eventsOfDaySelected || now;
        // Get a list of short streams for the selected date.
        buffPromise.push(this.streamCalendarService.getListEventsForDate(selectedDate, 1));
        const selectedMonth = this.streamCalendarService.calendarMonth || now;
        // Get a list of events (streams) for a specified date.
        buffPromise.push(this.streamCalendarService.getCalendarInfoForPeriod(selectedMonth, true));

        this.changeDetector.markForCheck();
        try {
            await Promise.all(buffPromise);
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    private async deleteDataStream(streamId: number): Promise<void> {
        this.errObj = null;
        this.changeDetector.markForCheck();
        try {
            // Delete a stream by its ID.
            await this.futureStreamHdlr.deleteDataStream(streamId);
            // Update the list of next and past streams.
            await this.loadFutureAndPastStreamsAndSchedule();
        } catch (error) {
            if (error instanceof HttpErrorResponse) {
                const value = HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0];
                this.errObj = { "key": CN_DELETE_STREAM, value };
            }
        } finally {
            this.changeDetector.markForCheck();
        }
    }
}
