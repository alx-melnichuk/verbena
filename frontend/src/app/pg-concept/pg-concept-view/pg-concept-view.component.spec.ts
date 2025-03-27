import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgConceptViewComponent } from './pg-concept-view.component';

describe('PgConceptViewComponent', () => {
  let component: PgConceptViewComponent;
  let fixture: ComponentFixture<PgConceptViewComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgConceptViewComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgConceptViewComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
