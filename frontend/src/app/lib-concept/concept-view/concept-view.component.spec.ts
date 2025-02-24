import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ConceptViewComponent } from './concept-view.component';

describe('ConceptViewComponent', () => {
  let component: ConceptViewComponent;
  let fixture: ComponentFixture<ConceptViewComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [ConceptViewComponent]
    });
    fixture = TestBed.createComponent(ConceptViewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
