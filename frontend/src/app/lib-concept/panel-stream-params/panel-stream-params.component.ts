import { ChangeDetectionStrategy, Component, Input, OnChanges, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

import { TranslatePipe, TranslateService } from '@ngx-translate/core';

import { DateTimeFormatPipe } from 'src/app/common/date-time-format.pipe';
import { DateTimeTimerComponent } from 'src/app/components/date-time-timer/date-time-timer.component';
import { TimeTrackingComponent } from 'src/app/components/time-tracking/time-tracking.component';

@Component({
    selector: 'app-panel-stream-params',
    exportAs: 'appPanelStreamParams',
    standalone: true,
    imports: [CommonModule, DateTimeFormatPipe, TranslatePipe, DateTimeTimerComponent, TimeTrackingComponent],
    templateUrl: './panel-stream-params.component.html',
    styleUrl: './panel-stream-params.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PanelStreamParamsComponent implements OnChanges {
    @Input()
    public countOfViewer: number | null | undefined;
    @Input()
    public isShowTimer: boolean | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input() // Time when stream should start.
    public start: Date | null | undefined;
    @Input() // Time when stream was started
    public started: Date | null | undefined;
    @Input() // Time when stream was stopped
    public stopped: Date | null | undefined;

    readonly formatDateTime: Intl.DateTimeFormatOptions = { dateStyle: 'long', timeStyle: 'short' };

    public date1: Date = new Date(Date.now() + (1 * 3600 * 24 + 3 * 3600 + 11 * 60 + 3) * 1000);

    public labelDays: Record<number, string> = this.createLabelDays(this.translate);

    constructor(private translate: TranslateService) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['locale']) {
            this.labelDays = this.createLabelDays(this.translate);
        }
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
