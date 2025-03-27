import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgConceptListComponent } from './pg-concept-list.component';

describe('PgConceptListComponent', () => {
  let component: PgConceptListComponent;
  let fixture: ComponentFixture<PgConceptListComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgConceptListComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgConceptListComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
