import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgConceptComponent } from './pg-concept.component';

describe('PgConceptComponent', () => {
  let component: PgConceptComponent;
  let fixture: ComponentFixture<PgConceptComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PgConceptComponent]
    });
    fixture = TestBed.createComponent(PgConceptComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
