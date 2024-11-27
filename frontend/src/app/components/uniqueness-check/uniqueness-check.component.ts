import {
  ChangeDetectionStrategy, ChangeDetectorRef, Component, Input, ViewEncapsulation 
} from '@angular/core';
import { CommonModule } from '@angular/common';

import { SpinnerComponent } from '../spinner/spinner.component';
import { debounceFn } from 'src/app/common/debounce';

export const UC_SPINNER_DIAMETER = 40;
export const UC_DEBOUNCE_DELAY = 1000;

export interface UniquenessCheck {
  isChecking: boolean;
  isUniquenessError: boolean;
  checkParameter(value: string | null | undefined): void;
}

export type CheckUniquenessFnType = (value: string | null | undefined) => Promise<boolean>;

@Component({
  selector: 'app-uniqueness-check',
  exportAs: 'appUniquenessCheck',
  standalone: true,
  imports: [CommonModule, SpinnerComponent],
  templateUrl: './uniqueness-check.component.html',
  styleUrl: './uniqueness-check.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class UniquenessCheckComponent implements UniquenessCheck {
  @Input()
  public debounceDelay: number = UC_DEBOUNCE_DELAY;
  @Input()
  public checkUniquenessFn: ((value: string | null | undefined) => Promise<boolean>) | null | undefined;

  // ** interface UniquenessCheck **
  public isChecking: boolean = false;
  public isUniquenessError: boolean = false;
  public checkParameter = debounceFn((value: string | null | undefined) => this.checkParameterInner(value), 1000);

  public spinnerDiameter = UC_SPINNER_DIAMETER;

  constructor(private changeDetectorRef: ChangeDetectorRef) {
  }

  // ** Public API **
  
  // ** Private API **
  
  private checkParameterInner(value: string | null | undefined): void {
    if (this.checkUniquenessFn == null) {
      return;
    }
    this.isUniquenessError = false;
    this.isChecking = true;
    this.changeDetectorRef.markForCheck();
    this.checkUniquenessFn(value)
      .then((response: boolean) => this.isUniquenessError = !response)
      .finally(() => {
        this.isChecking = false;
        this.changeDetectorRef.markForCheck();
      });
  }

}
