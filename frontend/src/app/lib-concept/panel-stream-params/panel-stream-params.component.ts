import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, Input, OnChanges, OnInit, SimpleChanges, ViewEncapsulation } from '@angular/core';

import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { TimeTrackingComponent } from 'src/app/components/time-tracking/time-tracking.component';

@Component({
    selector: 'app-panel-stream-params',
    exportAs: 'appPanelStreamParams',
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, TranslatePipe, TimeTrackingComponent],
    templateUrl: './panel-stream-params.component.html',
    styleUrl: './panel-stream-params.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamParamsComponent implements OnChanges, OnInit {
    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input() // The stream start time.
    public start: Date | null | undefined;
    @Input() // The time the stream began.
    public started: Date | null | undefined;
    @Input() // The time the stream began pausing.
    public paused: Date | null | undefined;
    @Input() // The time the stream stopped.
    public stopped: Date | null | undefined;

    public timerActive: boolean | null | undefined;
    public timerIsShow: boolean | null | undefined;
    public timerValue: number | null | undefined;

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'long', timeStyle: 'short' };

    public startVal: Date | null | undefined;

    // public date1: Date = new Date(Date.now() + (1 * 3600 * 24 + 3 * 3600 + 11 * 60 + 3) * 1000);
    // public date1: Date = new Date(Date.now() + (0 * 3600 * 24 + 0 * 3600 + 0 * 60 + 15) * 1000);
    // const value = Math.floor((Date.now() - this.startTime.getTime()) / 1000);

    public labelDays: Record<number, string> = this.createLabelDays(this.translate);

    constructor(private translate: TranslateService) {
        this.timerIsShow = true;
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['locale']) {
            this.labelDays = this.createLabelDays(this.translate);
        }
        if (!!changes['start'] || !!changes['started'] || !!changes['paused'] || !!changes['stopped']) {
            console.log(`(new Date()):${(new Date()).toISOString()}`); // #
        }
        if (!!changes['start'] && !!this.start) {
            this.startVal = this.start;
            this.timerActive = true;
            this.timerValue = Math.floor((Date.now() - this.start.getTime()) / 1000);
            console.log(`this.start  :${this.start.toISOString()}`); // #
        }
        if (!!changes['started'] && !!this.started) {
            this.timerActive = true;
            this.timerValue = Math.floor((Date.now() - this.started.getTime()) / 1000);
            console.log(`this.started:${this.started.toISOString()}`); // #
        }
        if (!!changes['paused'] && !!this.paused && !!this.started) {
            this.timerActive = false;
            this.timerValue = Math.floor((this.paused.getTime() - this.started.getTime()) / 1000);
            console.log(`this.paused :${this.paused.toISOString()}`); // #
        }
        if (!!changes['stopped'] && !!this.stopped && !!this.started) {
            this.timerActive = false;
            this.timerValue = Math.floor((this.stopped.getTime() - this.started.getTime()) / 1000);
            console.log(`this.stopped:${this.stopped.toISOString()}`); // #
        }
    }

    ngOnInit(): void {
    }

    // ** Private API **

    private createLabelDays(translate: TranslateService): Record<number, string> {
        const result: Record<number, string> = {};
        for (let idx = -1; idx <= 20; idx++) {
            const key = `panel-stream-params.time-tracking.${idx}`;
            if (translate.instant(key) != key) {
                result[idx] = key;
            }
        }
        return result;
    }
}
