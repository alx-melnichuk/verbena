import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, computed, Input, OnChanges, OnDestroy, signal, SimpleChanges, ViewEncapsulation
} from '@angular/core';
import { TranslatePipe } from '@ngx-translate/core';

import { LOCALE_UK } from 'src/app/common/constants';
import { BooleanUtil } from 'src/app/utils/boolean.util';

const LABEL_DAYS_DEF = 'days';
const FILL_SYMBOL_ZERO = '0';
const FILL_SYMBOL_EMPTY = '';
const INTERVAL = 1000;

let idx = 0;

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
    public id = 'timeTrackingId_' + (++idx);
    @Input()
    public initValue: number | null | undefined;
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

    public isHideDaysVal: boolean | null = null; // Binding attribute "isHideDays".
    public isHideLeadingZeroVal: boolean | null = null; // Binding attribute "isHideLeadingZero".
    public isUnstopAtZeroVal: boolean | null = null; // Binding attribute "isUnstopAtZero".
    public innLabelDays: string = '';
    public labelDaysObj: Record<number, string> = this.getLabelDaysObj(null);

    public fillSymbol: string = FILL_SYMBOL_ZERO;

    // Writable signals for managing timer state.
    private expected: number = 0;
    private isFirstIteration: boolean = false;
    private timeTracking = signal(0); // Counting the number of seconds elapsed.
    private timeoutId: number | null = null;

    // Expose read-only versions for the template
    readonly timeValue = this.timeTracking.asReadonly();

    constructor() {
        // Effect for logging state changes (useful for debugging)
        // effect(() => { console.log(`this.timeValue: ${this.timeValue()}`); });
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['initValue']) {
            this.setTimeTracking(this.initValue || 0);
            if (this.isActive && !changes['isActive']) {
                this.startTimeTracking();
            }
        }
        if (!!changes['isActive']) {
            if (this.isActive) {
                this.startTimeTracking();
            } else {
                this.stopTimeTracking();
            }
        }
        if (!!changes['isHideDays']) {
            this.isHideDaysVal = !!BooleanUtil.init(this.isHideDays);
        }
        if (!!changes['isHideLeadingZero']) {
            this.isHideLeadingZeroVal = !!BooleanUtil.init(this.isHideLeadingZero);
            this.fillSymbol = this.isHideLeadingZeroVal ? FILL_SYMBOL_EMPTY : FILL_SYMBOL_ZERO;
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
        if (!!this.timeoutId) {
            return;
        }
        this.isFirstIteration = true;
        this.doIterate();
    }
    public doIterate() {
        this.updateTimeTracking();
        const currNow = Date.now();
        if (this.expected < currNow) {
            this.expected = currNow;
        }
        // Drift (ms) (positive for exceeded) compares the expected time to the current time.
        const delta = currNow - this.expected;
        // Set the next expected current time when the timeout fires.
        this.expected += INTERVAL;

        this.timeoutId = window.setTimeout(() => {
            this.doIterate();
        }, Math.max(0, INTERVAL - delta)); // Take drift into account.
    };

    /** Stop timer processing (calls to "setInterval()"). */
    public stopTimeTracking(): void {
        if (this.timeoutId != null) {
            window.clearInterval(this.timeoutId);
            this.timeoutId = null;
            this.expected = 0;
        }
        this.isFirstIteration = false;
    }

    public getLabelByKey(labelDaysObj: Record<number, string>, key: number, locale: string | null): string | undefined {
        let result: string | undefined;
        if (locale == LOCALE_UK) {
            const key2 = key % 100;
            result = (key2 < 20 ? labelDaysObj[key2] : labelDaysObj[key % 10]);
        } else {
            result = labelDaysObj[key > 1 ? -1 : 1]
        }
        return result || labelDaysObj[-1];
    }

    public padsStart(value: number, fillString: string): string {
        return value.toString().padStart(2, fillString);
    }

    // ** Computed signals for derived state **

    // Get the number of days the timer has been running.
    readonly getDays = computed(() => {
        return Math.floor(Math.abs(this.timeValue()) / 86400); // 3600 * 24 = 86400
    });
    /** Get the number of hours the timer has been running. */
    readonly getHours = computed(() => {
        return Math.floor((Math.abs(this.timeValue()) % 86400) / 3600); // 3600 * 24 = 86400
    });
    /** Get the number of minutes the timer has been running. */
    readonly getMinutes = computed(() => {
        return Math.floor((Math.abs(this.timeValue()) % 3600) / 60);
    });
    /** Get the number of seconds the timer has been running. */
    readonly getSeconds = computed(() => {
        return Math.abs(this.timeValue()) % 60;
    });

    readonly isCompleted = computed(() => this.timeValue() === 0);

    // ** Private API **

    private updateTimeTracking = () => {
        this.timeTracking.update((totalSeconds: number): number => {
            let delta = 0;
            if (!this.isFirstIteration && !this.isUnstopAtZeroVal && totalSeconds == 0) {
                this.stopTimeTracking();
            } else {
                delta = 1;
            }
            if (this.isFirstIteration) {
                this.isFirstIteration = false;
            }
            return totalSeconds + delta;
        });
    }

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
