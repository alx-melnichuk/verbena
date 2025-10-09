import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, computed, Input, OnChanges, OnDestroy, signal, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { TranslatePipe } from '@ngx-translate/core';

import { LOCALE_UK } from 'src/app/common/constants';
import { BooleanUtil } from 'src/app/utils/boolean.util';

const LABEL_DAYS_DEF = 'days_';

@Component({
    selector: 'app-time-tracking',
    standalone: true,
    exportAs: 'appTimeTracking',
    imports: [CommonModule, TranslatePipe],
    templateUrl: './time-tracking.component.html',
    styleUrl: './time-tracking.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class TimeTrackingComponent implements OnChanges, OnDestroy {
    @Input()
    public isActive: boolean | null | undefined;
    @Input()
    public isHideDays: string | boolean | null | undefined;
    @Input()
    public isHideLeadingZero: string | boolean | null | undefined;
    @Input()
    public isUnstopAtZero: string | boolean | null | undefined;
    @Input()
    public labelDays: string | Record<number, string> | null | undefined;
    @Input()
    public locale: string | null = null;
    @Input()
    public startTime: Date | null | undefined;

    public isHideDaysVal: boolean | null = null; // Binding attribute "isHideDays".
    public isHideLeadingZeroVal: boolean | null = null; // Binding attribute "isHideLeadingZero".
    public isUnstopAtZeroVal: boolean | null = null; // Binding attribute "isUnstopAtZero".
    public innLabelDays: string = '';
    public labelDaysObj: Record<number, string> = this.getLabelDaysObj(null);

    // Writable signals for managing timer state.
    private timeTracking = signal(0); // Counting the number of seconds elapsed.
    private intervalId: number | null = null;

    // Expose read-only versions for the template
    readonly timeValue = this.timeTracking.asReadonly();

    constructor() {
        // Effect for logging state changes (useful for debugging)
        // effect(() => { console.log(`this.timeValue: ${this.timeValue()}`); });
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['startTime']) {
            if (!!this.startTime) {
                const value = Math.floor((Date.now() - this.startTime.getTime()) / 1000);
                this.setTimeTracking(value);
            }
        }
        if (!!changes['isActive']) {
            if (!!this.startTime) {
                if (this.isActive) {
                    this.startTimeTracking();
                } else {
                    this.stopTimeTracking();
                }
            }
        }
        if (!!changes['isHideDays']) {
            this.isHideDaysVal = !!BooleanUtil.init(this.isHideDays);
        }
        if (!!changes['isHideLeadingZero']) {
            this.isHideLeadingZeroVal = !!BooleanUtil.init(this.isHideLeadingZero);
        }
        if (!!changes['isUnstopAtZero']) {
            this.isUnstopAtZeroVal = !!BooleanUtil.init(this.isUnstopAtZero);
        }
        if (!!changes['labelDays']) {
            this.labelDaysObj = this.getLabelDaysObj(this.labelDays);
        }
    }

    ngOnDestroy(): void {
        // Clean up interval when component is destroyed
        this.stopTimeTracking();
    }

    // ** Public API **

    /** Set the timer value. */
    public setTimeTracking(seconds: number): void {
        this.stopTimeTracking();
        this.timeTracking.set(seconds);
    }
    /** Start timer processing (calls "setInterval()"). */
    public startTimeTracking(): void {
        if (!!this.intervalId) {
            return;
        }
        this.intervalId = window.setInterval(() => {
            this.timeTracking.update((totalSeconds) => {
                let delta = 1;
                if (!this.isUnstopAtZeroVal && totalSeconds == 0) {
                    delta = 0;
                    this.stopTimeTracking();
                }
                return totalSeconds + delta;
            });
        }, 1000);
    }
    /** Stop timer processing (calls to "setInterval()"). */
    public stopTimeTracking(): void {
        if (this.intervalId != null) {
            window.clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }

    public getLabelByKey(labelDaysObj: Record<number, string>, key: number, locale: string | null): string | undefined {
        let result: string | undefined;
        if (locale == LOCALE_UK) {
            result = (key == 1 ? labelDaysObj[1] : (key < 20 ? labelDaysObj[key] : labelDaysObj[key % 10]));
        } else {
            result = labelDaysObj[key > 1 ? -1 : 1]
        }
        return result || labelDaysObj[-1];
    }

    // ** Computed signals for derived state **

    // Get the number of days the timer has been running.
    readonly getDays = computed(() => {
        const days = Math.floor(Math.abs(this.timeValue()) / (3600 * 24));
        return days;
    });

    // Get the number of hours the timer has been running.
    readonly getHours = computed(() => {
        const fillStr = this.isHideLeadingZeroVal ? '' : '0';
        const hours = Math.floor((Math.abs(this.timeValue()) % (3600 * 24)) / 3600);
        return `${hours.toString().padStart(2, fillStr)}`;
    });

    // Get the number of minutes the timer has been running.
    readonly getMinutes = computed(() => {
        const fillStr = this.isHideLeadingZeroVal ? '' : '0';
        const minutes = Math.floor((Math.abs(this.timeValue()) % 3600) / 60);
        return `${minutes.toString().padStart(2, fillStr)}`;
    });

    // Get the number of seconds the timer has been running.
    readonly getSeconds = computed(() => {
        const fillStr = this.isHideLeadingZeroVal ? '' : '0';
        const seconds = Math.abs(this.timeValue()) % 60;
        return `${seconds.toString().padStart(2, fillStr)}`;
    });

    readonly isCompleted = computed(() => this.timeValue() === 0);

    // ** Private API **

    private getLabelDaysObj(labelDays: string | Record<number, string> | null | undefined): Record<number, string> {
        const labelDaysObj: Record<number, string> = { [-1]: LABEL_DAYS_DEF };
        if (labelDays != null) {
            const typeName = typeof labelDays;
            if (typeName == 'object') {
                const labelDays2 = labelDays as Record<number, string>;
                for (const key in labelDays2) labelDaysObj[key] = labelDays2[key];
            } else if (typeName == 'string') {
                labelDaysObj[-1] = labelDays as string;
            }
        }
        return labelDaysObj;
    }
}
