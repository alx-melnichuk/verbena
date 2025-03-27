import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PgStreamEditComponent } from './pg-stream-edit.component';

describe('PgStreamEditComponent', () => {
  let component: PgStreamEditComponent;
  let fixture: ComponentFixture<PgStreamEditComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PgStreamEditComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PgStreamEditComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
