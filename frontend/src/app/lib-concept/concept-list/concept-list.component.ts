import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-concept-list',
  exportAs: 'appConceptList',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './concept-list.component.html',
  styleUrls: ['./concept-list.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class ConceptListComponent {

}
