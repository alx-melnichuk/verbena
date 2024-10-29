import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule } from '@ngx-translate/core';

@Component({
  selector: 'app-logotype',
  exportAs: 'appLogotype',
  standalone: true,
  imports: [CommonModule, TranslateModule],
  templateUrl: './logotype.component.html',
  styleUrls: ['./logotype.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class LogotypeComponent {
  @Input()
  public logo: string | null = null;
}
