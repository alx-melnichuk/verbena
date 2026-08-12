import { CommonModule } from "@angular/common";
import { ChangeDetectionStrategy, ChangeDetectorRef, Component, inject, Input, ViewEncapsulation } from "@angular/core";
import { CallbackDebounceFnType, debounceFn } from "../../common/debounce";
import { Spinner } from "../spinner/spinner";

export const UC_SPINNER_DIAMETER = 40;
export const UC_DEBOUNCE_DELAY = 1000;

export interface UniquenessCheck {
    isChecking: boolean;
    isUniquenessError: boolean;
    checkParameter(value: string | null | undefined): void;
}

export type CheckUniqueFnType = (value: string | null | undefined) => Promise<boolean>;

@Component({
    selector: "app-uniqueness-check",
    exportAs: "appUniquenessCheck",
    standalone: true,
    imports: [CommonModule, Spinner],
    templateUrl: "./uniqueness-check.html",
    styleUrl: "./uniqueness-check.scss",
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class UniquenessCheck implements UniquenessCheck {
    @Input()
    public debounceDelay: number = UC_DEBOUNCE_DELAY;
    @Input()
    public checkUniqueFn: ((value: string | null | undefined) => Promise<boolean>) | null | undefined;

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);

    // ** interface UniquenessCheck **

    public isChecking: boolean = false;
    public isUniquenessError: boolean = false;
    public checkParameter = (value: string | null | undefined): void => this.checkPrmt(value);

    // **

    public checkPrmt = debounceFn((value: string | null | undefined) => this.checkParameterInner(value), (this.debounceDelay || UC_DEBOUNCE_DELAY));

    public spinnerDiameter = UC_SPINNER_DIAMETER;

    // ** Public API **

    // ** Private API **

    private checkParameterInner: CallbackDebounceFnType = (value: string | null | undefined): void => {
        if (this.checkUniqueFn == null) {
            return;
        }
        this.isUniquenessError = false;
        this.isChecking = true;
        this.changeDetector.markForCheck();
        this.checkUniqueFn(value)
            .then((response: boolean) => this.isUniquenessError = !response)
            .finally(() => {
                this.isChecking = false;
                this.changeDetector.markForCheck();
            });
    }
}
