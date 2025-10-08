import { CommonModule } from '@angular/common';
import {
    ChangeDetectionStrategy, Component, computed, Input, OnChanges, OnDestroy, signal, SimpleChanges, ViewEncapsulation
} from '@angular/core';


@Component({
    selector: 'app-time-tracking',
    standalone: true,
    imports: [CommonModule],
    templateUrl: './time-tracking.component.html',
    styleUrl: './time-tracking.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class TimeTrackingComponent implements OnChanges, OnDestroy {
    @Input()
    public isActive: boolean | null | undefined;
    @Input()
    public startTime: Date | null | undefined;

    // Writable signals for managing timer state.
    private timeTracking = signal(0);
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
                this.setTimeTracking(0);
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
            this.timeTracking.update((oldSeconds) => {
                return oldSeconds + 1;
            });
        }, 1000);
    }
    /** Stop timer processing (calls to "setInterval()"). */
    public stopTimeTracking(): void {
        if (!!this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }

    // ** Computed signals for derived state **

    // Get the number of hours the timer has been running.
    readonly getHours = computed(() => {
        const hours = Math.floor((this.timeValue() % (3600 * 24)) / (3600));
        return `${hours.toString().padStart(2, '0')}`;
    });

    // Get the number of minutes the timer has been running.
    readonly getMinutes = computed(() => {
        const minutes = Math.floor((this.timeValue() % 3600) / 60);
        return `${minutes.toString().padStart(2, '0')}`;
    });

    // Get the number of seconds the timer has been running.
    readonly getSeconds = computed(() => {
        const seconds = this.timeValue() % 60;
        return `${seconds.toString().padStart(2, '0')}`;
    });

    // ** Private API **
}
