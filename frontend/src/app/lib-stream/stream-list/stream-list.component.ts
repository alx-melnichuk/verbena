import { ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule, KeyValue } from '@angular/common';
import { HttpErrorResponse } from '@angular/common/http';
import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { LocaleService } from 'src/app/common/locale.service';
import { SpinnerComponent } from 'src/app/components/spinner/spinner.component';
import { AlertService } from 'src/app/lib-dialog/alert.service';
import { DialogService } from 'src/app/lib-dialog/dialog.service';
import { ProfileDto } from 'src/app/lib-profile/profile-api.interface';
import { HttpErrorUtil } from 'src/app/utils/http-error.util';

import { PanelStreamCalendarComponent } from '../panel-stream-calendar/panel-stream-calendar.component';
import { PanelStreamEventComponent } from '../panel-stream-event/panel-stream-event.component';
import { PanelStreamInfoComponent } from '../panel-stream-info/panel-stream-info.component';
import { StreamCalendarService } from '../stream-calendar.service';
import { StreamListService } from '../stream-list.service';
import { StreamService } from '../stream.service';

@Component({
    selector: 'app-stream-list',
    standalone: true,
    imports: [CommonModule, TranslatePipe, SpinnerComponent, PanelStreamEventComponent, PanelStreamInfoComponent,
        PanelStreamCalendarComponent, DateTimeFormatPipe],
    templateUrl: './stream-list.component.html',
    styleUrl: './stream-list.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class StreamListComponent implements OnChanges, OnInit {
    @Input()
    public err: string | null = null;
    @Input()
    public profileDto: ProfileDto | null = null;

    @Output()
    readonly actionDuplicate: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionEdit: EventEmitter<number> = new EventEmitter();
    @Output()
    readonly actionDelete: EventEmitter<number> = new EventEmitter();

    public isRefreshPanelStreamEvent: boolean | null | undefined;

    readonly formatDate: Intl.DateTimeFormatOptions = { dateStyle: 'long' };

    constructor(
        private changeDetector: ChangeDetectorRef,
        private translateService: TranslateService,
        private dialogService: DialogService,
        private streamService: StreamService,
        private alertService: AlertService,
        public localeService: LocaleService,
        public streamListService: StreamListService,
        public streamCalendarService: StreamCalendarService,
    ) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['err'] && !!this.err) {
            this.alertService.hide();
            // this.alertService.showError(HttpErrorUtil.getMsgs(err as HttpErrorResponse)[0], 'stream_list.error_delete_stream');
        }
    }


    async ngOnInit(): Promise<void> {
        await this.loadFutureAndPastStreamsAndSchedule();
    }

    // ** Public API **

    // ** "Streams Calendar" panel-stream-calendar **

    public async getCalendarInfoForPeriod(calendarStart: Date): Promise<void> {
        const buffPromise: Promise<unknown>[] = [];
        // Get a list of events (streams) for a specified date.
        buffPromise.push(this.streamCalendarService.getCalendarInfoForPeriod(calendarStart, false));
        if (this.streamCalendarService.isShowEvents(calendarStart)) {
            buffPromise.push(this.getListEventsForDate(this.streamCalendarService.eventsOfDaySelected, 1));
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

    public async getListEventsForDate(selectedDate: Date | null, pageNum: number): Promise<void> {
        try {
            // Get the next page of the list of short streams for the selected date.
            await this.streamCalendarService.getListEventsForDate(selectedDate, pageNum)
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** "Future Stream" and "Past Stream" panel-stream-info **

    public async searchInfoNextFutureStream(): Promise<void> {
        try {
            // Get the next page of the "Future Stream".
            await this.streamListService.searchNextFutureStream();
            // Refresh view for "panel-stream-event".
            this.isRefreshPanelStreamEvent = !this.isRefreshPanelStreamEvent ? true : undefined;
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    public async searchInfoNextPastStream(): Promise<void> {
        try {
            // Get the next page of "Past Stream".
            await this.streamListService.searchNextPastStream();
            // Refresh view for "panel-stream-event".
            this.isRefreshPanelStreamEvent = !this.isRefreshPanelStreamEvent ? true : undefined;
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    // ** **

    public doActionDuplicate(streamId: number): void {
        this.actionDuplicate.emit(streamId);
        //     this.streamService.redirectToStreamCreationPage(streamId);
    }

    public doActionEdit(streamId: number): void {
        this.actionEdit.emit(streamId);
        //     this.streamService.redirectToStreamEditingPage(streamId);
    }

    public redirectToStreamView(streamId: number): void {
        this.streamService.redirectToStreamViewPage(streamId);
    }

    public async doActionDelete(info: { id: number, title: string }): Promise<void> {
        this.alertService.hide();
        if (!info || !info.id) {
            return Promise.resolve();
        }
        const message = this.translateService.instant('stream_list.sure_you_want_delete_stream', { title: info.title });
        const res = await this.dialogService.openConfirmation(message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' });
        if (!!res) {
            this.actionDelete.emit(info.id);
        }
    }
    /*public async actionDelete0(info: { id: number, title: string }): Promise<void> {
        this.alertService.hide();
        if (!info || !info.id) {
            return Promise.resolve();
        }
        const message = this.translateService.instant('stream_list.sure_you_want_delete_stream', { title: info.title });
        const res = await this.dialogService.openConfirmation(message, '', { btnNameCancel: 'buttons.no', btnNameAccept: 'buttons.yes' });
        if (!!res) {
            await this.deleteDataStream(info.id);
        }
    }*/

    // ** Private API **

    private async loadFutureAndPastStreamsAndSchedule(): Promise<void> {
        // Clear array of "Future Stream"
        this.streamListService.clearFutureStream();
        // Clear array of "Past Stream"
        this.streamListService.clearPastStream();

        const buffPromise: Promise<unknown>[] = [];
        const now = new Date();
        now.setHours(0, 0, 0, 0);

        // Get the next page of the "Future Stream".
        buffPromise.push(this.streamListService.searchNextFutureStream());
        // Get the next page of "Past Stream".
        buffPromise.push(this.streamListService.searchNextPastStream());
        const selectedDate = this.streamCalendarService.eventsOfDaySelected || now;
        // Get a list of short streams for the selected date.
        buffPromise.push(this.streamCalendarService.getListEventsForDate(selectedDate, 1));
        const selectedMonth = this.streamCalendarService.calendarMonth || now;
        // Get a list of events (streams) for a specified date.
        buffPromise.push(this.streamCalendarService.getCalendarInfoForPeriod(selectedMonth, true));

        try {
            await Promise.all(buffPromise);
        } finally {
            this.changeDetector.markForCheck();
        }
    }

    private async deleteDataStream(streamId: number): Promise<void> {
        this.alertService.hide();
        if (!streamId) {
            return Promise.reject();
        }
        let isRefres = false;
        try {
            await this.streamService.deleteStream(streamId);
            isRefres = true;
        } catch (error) {
            this.alertService.showError(HttpErrorUtil.getMsgs(error as HttpErrorResponse)[0], 'stream_list.error_delete_stream');
            throw error;
        } finally {
            this.changeDetector.markForCheck();
            if (isRefres) {
                setTimeout(() => this.loadFutureAndPastStreamsAndSchedule(), 0);
            }
        }
    }
}
